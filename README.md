# example-restapi

## Prerequisites

1. Install bpf-linker: `cargo install bpf-linker`

## Build eBPF

```bash
cargo xtask build-ebpf
```

To perform a release build you can use the `--release` flag.
You may also change the target architecture with the `--target` flag.

## Build Userspace

```bash
cargo build
```

## Build eBPF and Userspace

```bash
cargo xtask build
```

## Run

```bash
RUST_LOG=info cargo xtask run
```

```bash
curl -X POST -d "{\"ip\": \"8.8.8.8\"}" -H "Content-Type: Application/json" -H "Authorization: Bearer APITOKEN" http://127.0.0.1:5000/api/v1/block

dig @8.8.8.8 toto

```

Results in

```bash
[2024-06-19T17:55:38Z INFO  example_restapi::controllers::blocklist] Adding IP 8.8.8.8 to blocklist
[2024-06-19T17:55:39Z INFO  example_restapi] DROPPING SRC: 8.8.8.8
[2024-06-19T17:55:44Z INFO  example_restapi] DROPPING SRC: 8.8.8.8
[2024-06-19T17:55:49Z INFO  example_restapi] DROPPING SRC: 8.8.8.8
```