use crate::{not_modified_response, ServerError};
use axum::{response::IntoResponse, Json};
use land_core::{
    background,
    gateway::DATA,
    workerinfo::sync::{SyncRequest, SyncResponse},
};

/// deploys returns the current deploys data.
pub async fn deploys() -> Result<Json<SyncResponse>, ServerError> {
    let data = DATA.lock().await.clone();
    let sync_resp = SyncResponse { data };
    Ok(Json(sync_resp))
}

/// deploys_post is the endpoint for worker to post the sync data.
pub async fn deploys_post(Json(j): Json<SyncRequest>) -> Result<impl IntoResponse, ServerError> {
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

    let data = DATA.lock().await.clone();
    if data.checksum == j.checksum {
        return Ok(not_modified_response().into_response());
    }
    let sync_resp = SyncResponse { data };
    Ok(Json(sync_resp).into_response())
}
