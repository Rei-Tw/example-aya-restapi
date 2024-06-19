
use std::iter::once;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};

use anyhow::Context;
use clap::Parser;
use log::{debug, info, warn};
use tokio::signal;

use aya::{
    include_bytes_aligned,
    maps::MapData,
    programs::{Xdp, XdpFlags},
    Ebpf,
};
use aya_log::EbpfLogger;

use axum::extract::FromRef;
use axum::{
    routing::{delete, post},
    Router,
};

use hyper::header::AUTHORIZATION;
use tower::ServiceBuilder;

use tower_http::{
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultMakeSpan, TraceLayer},
    validate_request::ValidateRequestHeaderLayer,
};

mod controllers;
mod error;
mod models;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

// Axum states
#[derive(Clone)]
pub struct AppState {
    blocklist_map_state: BlocklistMapState,
}

#[derive(Clone)]
pub struct BlocklistMapState {
    pub blocklist_map: Arc<RwLock<aya::maps::HashMap<MapData, u32, u32>>>,
}

impl FromRef<AppState> for BlocklistMapState {
    fn from_ref(app_state: &AppState) -> BlocklistMapState {
        app_state.blocklist_map_state.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/example-restapi"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/example-restapi"
    ))?;
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp = bpf.program_mut("example_restapi").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // https://docs.rs/axum/latest/axum/extract/struct.State.html
    let state = AppState {
        blocklist_map_state: BlocklistMapState {
            blocklist_map: Arc::new(RwLock::new(aya::maps::HashMap::try_from(
                bpf.take_map("BLOCKLIST").unwrap(),
            )?)),
        },
    };

    // Construct your routes (get, post, delete, put)
    let blocklist_router = Router::new()
        .route("/block", post(controllers::blocklist::add))
        .route("/block/:ip", delete(controllers::blocklist::remove))
        .with_state(state);

    // nest allows to "collapse" routes. eg: /block will be /api/v1/block
    let app = Router::new().nest("/api/v1", blocklist_router).layer(
        // Logs part, CORS and API Token
        ServiceBuilder::new()
            .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_body_chunk(())
                    .on_eos(())
                    .on_failure(()),
            )
            .layer(CorsLayer::new().allow_origin(Any))
            .layer(ValidateRequestHeaderLayer::bearer("APITOKEN")),
    );

    let addr = SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0).octets(), 5000));

    // Important to implement TLS
    info!("Starting axum server...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Exiting...");

    Ok(())
}

// Used for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
