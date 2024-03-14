use crate::{not_modified_response, ServerError};
use axum::{response::IntoResponse, Json};
use land_core::{background, workerinfo::sync::SyncRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploysResponse {
    pub data: land_kernel::cron::ConfData,
}

/// deploys returns the current deploys data.
pub async fn deploys() -> Result<Json<DeploysResponse>, ServerError> {
    let data = land_kernel::cron::get_deploys().await;
    let sync_resp = DeploysResponse { data };
    Ok(Json(sync_resp))
}

/// deploys_post is the endpoint for worker to post the sync data.
pub async fn deploys_post(Json(j): Json<SyncRequest>) -> Result<impl IntoResponse, ServerError> {
    println!("deploys_post: {:?}", j);
    // handle deploy result
    if !j.deploys.is_empty() {
        let _ = background::send_updater_task(j.ip.ip.clone(), j.deploys.clone()).await;
    }
    // refresh online status
    let hostname = j.ip.hostname.clone().unwrap_or_default();
    let info_json = serde_json::to_string(&j.ip)?;
    let _ = land_dao::worker::update(
        &j.ip.ip,
        &hostname,
        &info_json,
        "",
        land_dao::worker::Status::Online,
    )
    .await?;

    let data = land_kernel::cron::get_deploys().await;
    if data.checksum == j.checksum {
        return Ok(not_modified_response().into_response());
    }
    let sync_resp = DeploysResponse { data };
    Ok(Json(sync_resp).into_response())
}
