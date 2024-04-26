use super::ServerError;
use crate::deployer::TaskValue;
use anyhow::Result;
use axum::routing::get;
use axum::{response::IntoResponse, routing::post, Json, Router};
use land_common::IPInfo;
use land_dao::deployment::DeployStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// router returns the router for the worker api
pub fn router() -> Result<Router> {
    let app = Router::new()
        .route("/alive", post(alive))
        .route("/deploys", get(deploys));
    Ok(app)
}

/// alive is the handler for the /alive endpoint
async fn alive(Json(p): Json<Request>) -> Result<impl IntoResponse, ServerError> {
    let ipinfo = p.ip;
    let ip = ipinfo.ip.clone();
    let ipcontent = serde_json::to_string(&ipinfo)?;
    land_dao::worker::update_online(
        ipinfo.ip.clone(),
        "global".to_string(),
        ipinfo.hostname.unwrap_or("unknown".to_string()),
        ipcontent,
        String::new(),
        land_dao::worker::Status::Online,
    )
    .await?;

    tokio::spawn(async move {
        for (task_id, task_result) in p.tasks {
            land_dao::deployment::update_task_result(task_id, ip.clone(), task_result)
                .await
                .unwrap();
        }
    });

    let mut tasks_conf = vec![];
    let tasks =
        land_dao::deployment::list_tasks_by_ip(ipinfo.ip.clone(), Some(DeployStatus::Deploying))
            .await?;
    for task in tasks {
        tasks_conf.push(task.content);
    }
    if !tasks_conf.is_empty() {
        info!(ip = ipinfo.ip, "Alive with {} tasks", tasks_conf.len())
    }
    Ok(Json(tasks_conf))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub ip: IPInfo,
    pub tasks: HashMap<String, String>,
}

/// deploys is the handler for the /deploys endpoint
async fn deploys() -> Result<impl IntoResponse, ServerError> {
    let dps = land_dao::deployment::list_by_status(DeployStatus::Success).await?;
    let mut tasks = vec![];
    let (domain, _) = land_dao::settings::get_domain_settings().await?;
    let storage_settings = land_dao::settings::get_storage().await?;
    for dp in dps {
        let task = TaskValue {
            user_uuid: dp.user_uuid.clone(),
            project_uuid: dp.project_uuid.clone(),
            domain: format!("{}.{}", dp.domain, domain),
            download_url: storage_settings.build_url(&dp.storage_path)?,
            wasm_path: dp.storage_path.clone(),
            task_id: dp.task_id.clone(),
            checksum: dp.storage_md5,
        };
        tasks.push(task);
    }
    let content = serde_json::to_vec(&tasks)?;
    let checksum = format!("{:x}", md5::compute(content));
    Ok(Json(DeploysResp { checksum, tasks }))
}

#[derive(Serialize, Deserialize)]
struct DeploysResp {
    checksum: String,
    tasks: Vec<TaskValue>,
}
