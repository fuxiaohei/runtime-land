use crate::deployer::waiting::{set_failed, set_success};
use anyhow::{anyhow, Result};
use land_dao::{
    deploy_task,
    deploys::{self, Status},
    models::deployment,
};
use tracing::{debug, info, instrument, warn};

/// init_review starts handling waiting deploy tasks
pub async fn init_review() {
    debug!("deployer init_review");
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match review().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("deployer review handle error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

#[instrument("[DEPLOY-REVIEW]")]
async fn review() -> Result<()> {
    let deploy_data = deploys::list_by_deploy_status(Status::Deploying).await?;
    if deploy_data.is_empty() {
        return Ok(());
    }
    info!("Review: {}", deploy_data.len());
    for dp in deploy_data.iter() {
        match handle_one(dp).await {
            Ok(_) => {}
            Err(e) => {
                set_failed(dp.id, dp.project_id, e.to_string().as_str()).await?;
                warn!(dp_id = dp.id, "deployer waiting handle error: {:?}", e);
            }
        }
    }
    Ok(())
}

async fn handle_one(dp: &deployment::Model) -> Result<()> {
    debug!("Handle review: {}", dp.id);
    let tasks = deploy_task::list(None, None, Some(dp.task_id.clone())).await?;
    if tasks.len() as i32 != dp.total_count {
        return Err(anyhow!("Task count not match"));
    }
    let mut success_count = 0;
    let mut failed_message = "".to_string();
    let mut done_count = 0;
    for task in tasks.iter() {
        // 1. task is still doing, skip review this task
        if task.status == deploy_task::Status::Doing.to_string() {
            continue;
        }

        // 1.1 task is not doing, mean task is success or failed, must be done
        done_count += 1;

        // 2. task is success
        if task.status == deploy_task::Status::Success.to_string() {
            debug!(
                dp_id = dp.id,
                ip = task.worker_ip,
                task_id = dp.task_id,
                "task success"
            );
            success_count += 1;
            continue;
        }

        // 3. task is failed
        if task.status == deploy_task::Status::Failed.to_string() {
            debug!(
                dp_id = dp.id,
                ip = task.worker_ip,
                task_id = dp.task_id,
                "task failed: {}",
                task.message,
            );
            failed_message.clone_from(&task.message);
            continue;
        }
    }
    // 4. if all tasks are done, update deploy status
    if done_count == tasks.len() as i32 {
        if done_count != success_count {
            info!(dp_id = dp.id, "review failed: {:?}", failed_message);
            set_failed(dp.id, dp.project_id, &failed_message).await?;
            return Ok(());
        }
        info!(dp_id = dp.id, "review success");
        set_success(dp.id, dp.project_id).await?;
    } else {
        info!(dp_id = dp.id, "review not done");
    }
    Ok(())
}
