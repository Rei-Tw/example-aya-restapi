use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiBody {
    pub ip: Ipv4Addr,
}
