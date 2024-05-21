use super::ServerError;
use anyhow::Result;
use axum::extract::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use axum::{response::IntoResponse, routing::post, Json, Router};
use http::StatusCode;
use land_common::IPInfo;
use land_dao::confs::TaskValue;
use land_dao::deployment::DeployStatus;
use land_dao::user::TokenUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// router returns the router for the worker api
pub fn router() -> Result<Router> {
    let app = Router::new()
        .route("/alive", post(alive))
        .route("/alive2", post(alive2))
        .route("/deploys", get(deploys))
        .route("/envs", get(envs))
        .route_layer(middleware::from_fn(middleware));
    Ok(app)
}

pub async fn middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_token = request
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token = auth_token.trim_start_matches("Bearer ");
    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = land_dao::user::get_token_by_value(token, Some(TokenUsage::Worker))
        .await
        .map_err(|err| {
            info!("Token error: {}", err);
            StatusCode::UNAUTHORIZED
        })?;
    if token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliveRequest {
    pub ip: IPInfo,
    pub tasks: HashMap<String, String>,
    pub envs_conf_md5: Option<String>,
}

/// alive is the handler for the /alive endpoint
async fn alive(Json(p): Json<AliveRequest>) -> Result<impl IntoResponse, ServerError> {
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
pub struct AliveResponse {
    pub tasks: Vec<String>,
    pub envs_conf_md5: Option<String>,
}

async fn alive2(Json(p): Json<AliveRequest>) -> Result<impl IntoResponse, ServerError> {
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
    let mut resp = AliveResponse {
        tasks: tasks_conf,
        envs_conf_md5: None,
    };
    let envs_local = land_dao::envs::ENV_WORKER_LOCAL.lock().await;
    if !envs_local.md5.is_empty() {
        resp.envs_conf_md5 = Some(envs_local.md5.clone());
    }
    Ok(Json(resp))
}

/// deploys is the handler for the /deploys endpoint
async fn deploys() -> Result<impl IntoResponse, ServerError> {
    let dps = land_dao::deployment::list_by_status(DeployStatus::Success).await?;
    let mut tasks = vec![];
    let (domain, _, service_name) = land_dao::settings::get_domain_settings().await?;
    let storage_settings = land_dao::settings::get_storage().await?;
    for dp in dps {
        let task_value = TaskValue::new(&dp, &storage_settings, &domain, &service_name)?;
        tasks.push(task_value);
    }
    let content = serde_json::to_vec(&tasks)?;
    let checksum = format!("{:x}", md5::compute(content));
    Ok(Json(DeploysResp { checksum, tasks }))
}

async fn envs() -> Result<impl IntoResponse, ServerError> {
    let env_local = land_dao::envs::ENV_WORKER_LOCAL.lock().await;
    Ok(Json(env_local.clone()))
}

#[derive(Serialize, Deserialize)]
struct DeploysResp {
    checksum: String,
    tasks: Vec<TaskValue>,
}
