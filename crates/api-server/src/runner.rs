use crate::confs::{ConfData, CONFS};
use crate::AppError;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub runner_token: String,
    pub confs_md5: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub confs: Option<ConfData>,
    pub is_modified: bool,
}

pub async fn sync(Json(payload): Json<SyncRequest>) -> Result<Json<SyncResponse>, AppError> {
    debug!("request:{:?}", payload);

    let confs = CONFS.lock().await;
    if !payload.confs_md5.is_empty() && payload.confs_md5 == confs.routes_md5 {
        return Ok(Json(SyncResponse {
            confs: None,
            is_modified: false,
        }));
    }

    let resp = SyncResponse {
        confs: Some(confs.clone()),
        is_modified: true,
    };
    Ok(Json(resp))
}
