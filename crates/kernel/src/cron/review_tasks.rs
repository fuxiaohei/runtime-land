use anyhow::Result;
use land_dao::{deploy_task, deployment, worker};
use std::collections::HashMap;
use tracing::{debug, instrument};

#[instrument["[RW]"]]
pub async fn cron() -> Result<()> {
    // get living workers
    let workers = worker::list_online().await?;
    let living_ips: HashMap<String, bool> = workers.iter().map(|w| (w.ip.clone(), true)).collect();

    // list deploying deploment
    let ds = deployment::list_deploying().await?;
    for d in ds {
        if d.task_id.is_empty() {
            // FIXME: Why task id is empty
            continue;
        }
        let tasks = deploy_task::list_by_task_id(d.id, d.task_id).await?;
        if tasks.is_empty() {
            // FIXME: why no tasks for each ip?
            continue;
        }
        let mut finish_count = 0;
        let total_count = tasks.len();
        for t in tasks {
            if t.status != deploy_task::Status::Pending.to_string() {
                debug!(
                    ip = t.ip,
                    task_id = t.task_id,
                    "already status: {}",
                    t.status
                );
                finish_count += 1;
                continue;
            }
            if !living_ips.contains_key(&t.ip) {
                // the task ip is not online, make the task failed with ip not online message
                deploy_task::update_pending(
                    t.ip.clone(),
                    t.task_id.clone(),
                    deploy_task::Status::Failed,
                    "ip not online".to_string(),
                )
                .await?;
                debug!(ip = t.ip, task_id = t.task_id, "ip not online");
                finish_count += 1;
                continue;
            }
        }
        if finish_count == total_count {
            // all tasks are finished
            deployment::mark_status(
                d.id,
                deployment::DeployStatus::Success,
                "".to_string(),
                None,
                None,
            )
            .await?;
            debug!(dp_id = d.id, "all tasks are finished");
            continue;
        }
        debug!(dp_id = d.id, "waiting for tasks");
    }
    Ok(())
}
