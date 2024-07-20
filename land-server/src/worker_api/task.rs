use super::{response_ok, JsonError};
use axum::{extract::Query, response::IntoResponse, Json};
use land_dao::deploy_task;
use land_vars::Task;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Deserialize, Debug)]
pub struct IPQuery {
    ip: String,
}

type TaskResponse = HashMap<String, String>;

/// handle /worker-api/task
pub async fn handle(
    Query(q): Query<IPQuery>,
    Json(j): Json<TaskResponse>,
) -> Result<impl IntoResponse, JsonError> {
    if !j.is_empty() {
        for (task_id, res) in j.iter() {
            if res == "success" {
                deploy_task::set_success(q.ip.clone(), task_id.clone()).await?;
                info!(ip = q.ip, "Task {} success", task_id);
            } else {
                deploy_task::set_failed(q.ip.clone(), task_id.clone(), res.to_string()).await?;
                warn!(ip = q.ip, "Task {} failed: {}", task_id, res);
            }
        }
    }
    let models = deploy_task::list(Some(q.ip), Some(deploy_task::Status::Doing)).await?;
    if models.is_empty() {
        return Ok(response_ok(vec![], None));
    }
    let tasks: Vec<Task> = models.iter().map(Task::new).collect();
    Ok(response_ok(tasks, None))
}
