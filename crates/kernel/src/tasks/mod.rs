use anyhow::Result;
use land_dao::deploy_task;
use std::collections::HashMap;
use tracing::debug;

mod deploy;

/// DeployTask is the task for deployment
#[derive(Debug)]
pub struct DeployTask {
    pub project_id: i32,
    pub deploy_id: i32,     // deployment id is major priority
    pub playground_id: i32, // playground id is not zero and deploy_id must be zero at the same time
}

pub async fn init() -> Result<()> {
    deploy::init().await?;
    Ok(())
}

/// send_deploy_task is the function to send deploy task to background channel
pub async fn send_deploy_task(t: DeployTask) {
    debug!("Send_deploy_task: {:?}", t);
    deploy::DEPLOY_TASK_SENDER
        .get()
        .unwrap()
        .send(t)
        .await
        .unwrap();
}

pub async fn refresh_deploy_results(ip: String, data: HashMap<String, String>) {
    for (task_id, status) in data {
        let result_status = match status == "ok" {
            true => deploy_task::Status::Success,
            false => deploy_task::Status::Failed,
        };
        debug!(
            "Refresh_deploy_results:{:?}, {:?}, {:?}, {:?}",
            ip.clone(),
            task_id,
            result_status.to_string(),
            status
        );
        deploy_task::update_pending(ip.clone(), task_id, result_status, status.clone())
            .await
            .unwrap();
    }
}
