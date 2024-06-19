use std::net::Ipv4Addr;

use axum::{
    extract::{Path, State},
    Json,
};

use log::info;

use crate::{error::ApiError, models::ip::ApiBody, BlocklistMapState};

pub async fn add(
    State(blocklist_map_state): State<BlocklistMapState>,
    Json(data): Json<ApiBody>,
) -> Result<(), ApiError> {
    let blocklist = blocklist_map_state.blocklist_map;

    info!("Adding IP {:?} to blocklist", data.ip);

    let mut blocklist = blocklist.write().unwrap();

    match blocklist.insert(u32::from(data.ip), 0, 0) {
        Ok(_) => Ok(()),
        Err(_) => Err(ApiError::InternalServerError),
    }
}

pub async fn remove(
    State(blocklist_map_state): State<BlocklistMapState>,
    Path(ip): Path<Ipv4Addr>,
) -> Result<(), ApiError> {
    let blocklist = blocklist_map_state.blocklist_map;

    info!("Removing IP {:?} to blocklist", ip);

    let mut blocklist = blocklist.write().unwrap();

    match blocklist.remove(&u32::from(ip)) {
        Ok(_) => Ok(()),
        Err(_) => Err(ApiError::InternalServerError),
    }
}
