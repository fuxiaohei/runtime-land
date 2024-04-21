use super::ServerError;
use anyhow::Result;
use axum::{response::IntoResponse, routing::post, Json, Router};
use land_common::IPInfo;
use land_dao::deployment::DeployStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// router returns the router for the worker api
pub fn router() -> Result<Router> {
    let app = Router::new().route("/alive", post(alive));
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
        land_dao::deployment::list_tasks_by_ip(ipinfo.ip, Some(DeployStatus::Deploying)).await?;
    for task in tasks {
        tasks_conf.push(task.content);
    }
    Ok(Json(tasks_conf))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub ip: IPInfo,
    pub tasks: HashMap<String, String>,
}
