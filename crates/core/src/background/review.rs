use anyhow::Result;
use land_dao::{deploy_task, deployment};
use tracing::{error, info, instrument, warn};

/// init is the function to review background tasks
pub fn init() {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            match do_review().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Review error: {:?}", e);
                }
            }
        }
    });
}

#[instrument("[ReW]")]
async fn do_review() -> Result<()> {
    let deploys = deployment::list_deploying().await?;
    if deploys.is_empty() {
        return Ok(());
    }
    for deploy in deploys {
        let task_id = deploy.task_id;
        if task_id.is_empty() {
            warn!(dp_id = deploy.id, "task_id is empty");
            deployment::mark_status(
                deploy.id,
                deployment::DeployStatus::Failed,
                "task_id is empty".to_string(),
                None,
                None,
            )
            .await?;
            continue;
        }
        let tasks = deploy_task::list_by_task_id(deploy.id, task_id).await?;
        if tasks.is_empty() {
            warn!(dp_id = deploy.id, "no tasks");
            deployment::mark_status(
                deploy.id,
                deployment::DeployStatus::Failed,
                "no tasks".to_string(),
                None,
                None,
            )
            .await?;
            continue;
        }
        let total_count = tasks.len();
        let mut success_count = 0;
        let mut end_count = 0;
        for task in tasks {
            if task.status != deploy_task::Status::Pending.to_string() {
                end_count += 1;
            }
            if task.status == deploy_task::Status::Success.to_string() {
                success_count += 1;
            }
        }
        if total_count == success_count {
            // all are success, set deploy success
            deployment::mark_status(
                deploy.id,
                deployment::DeployStatus::Success,
                "ok".to_string(),
                None,
                None,
            )
            .await?;
            info!(dp_id = deploy.id, "success");
            continue;
        }
        if total_count == end_count {
            // all are end, but not all are success, set deploy failed
            deployment::mark_status(
                deploy.id,
                deployment::DeployStatus::Failed,
                "not all success".to_string(),
                None,
                None,
            )
            .await?;
            info!(dp_id = deploy.id, "failed");
            continue;
        }
    }
    Ok(())
}
