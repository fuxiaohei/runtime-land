use anyhow::Result;
use land_dao::models::deployment::Model as DeploymentModel;
use tracing::{debug, debug_span, info, warn, Instrument};

/// run_tasks runs the deploying tasks
pub async fn run_tasks() -> Result<()> {
    // 1.read deploying tasks from db
    let dps =
        land_dao::deployment::list_by_status(land_dao::deployment::DeployStatus::Deploying).await?;
    if dps.is_empty() {
        // debug!("No deploying tasks");
        return Ok(());
    }
    debug!("Found {} deploying tasks", dps.len());

    // 2. if no workers online, set all dps as failed
    let workers = land_dao::worker::list_online().await?;
    if workers.is_empty() {
        for dp in dps {
            land_dao::deployment::set_failed(dp.id, dp.project_id, "No online workers".to_string())
                .await?;
            warn!(
                id = dp.id,
                domain = dp.domain,
                "Deployment failed, no online workers"
            );
        }
        return Ok(());
    }

    // 3. handle each task
    for dp in dps {
        tokio::spawn(async move {
            if let Err(e) = handle_deploy(&dp)
                .instrument(debug_span!("[DEPLOY-2]", dp = dp.id))
                .await
            {
                let _ = land_dao::deployment::set_failed(dp.id, dp.project_id, e.to_string()).await;
                warn!("Handle deploy failed: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_deploy(dp: &DeploymentModel) -> Result<()> {
    let tasks = land_dao::deployment::list_tasks_by_taskid(dp.task_id.clone()).await?;
    if tasks.is_empty() {
        land_dao::deployment::set_failed(dp.id, dp.project_id, "No tasks found".to_string())
            .await?;
        warn!(
            id = dp.id,
            domain = dp.domain,
            "Deployment failed, no tasks found"
        );
        return Ok(());
    }

    // if all tasks are done, set dp as deployed
    let total_count = tasks.len();
    let mut success_count = 0;
    let mut final_count = 0;
    for task in tasks {
        // if task is deploying, skip
        if task.deploy_status == land_dao::deployment::DeployStatus::Deploying.to_string() {
            continue;
        }
        final_count += 1;
        if task.deploy_status == land_dao::deployment::DeployStatus::Success.to_string() {
            success_count += 1;
        }
    }

    // if total_count == success_count, set dp as deployed
    if total_count == success_count {
        land_dao::deployment::set_success(dp.id, dp.project_id).await?;
        info!(id = dp.id, domain = dp.domain, "Deployment success");
    } else if final_count == total_count {
        land_dao::deployment::set_failed(dp.id, dp.project_id, "Some tasks failed".to_string())
            .await?;
        warn!(
            id = dp.id,
            domain = dp.domain,
            "Deployment failed, some tasks failed"
        );
    } else {
        debug!(id = dp.id, domain = dp.domain, "Deployment still running");
    }

    Ok(())
}
