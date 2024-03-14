use anyhow::{anyhow, Result};
use land_dao::deploy_task::{self, Status};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, instrument, warn};

#[derive(Debug)]
pub struct Task {
    pub ip: String,
    pub res: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Updater {
    tx: mpsc::Sender<Task>,
}

impl Updater {
    pub async fn send(&self, task: Task) -> Result<()> {
        self.tx.send(task).await.map_err(|e| anyhow!(e))
    }
}

pub fn init() {
    let (tx, mut rx) = mpsc::channel(32);
    super::UPDATER_SENDER.set(Updater { tx }).unwrap();

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            handle_task(&task).await;
        }
    });
}

#[instrument("[UP]", skip(task))]
async fn handle_task(task: &Task) {
    for (task_id, status) in &task.res {
        if task_id.is_empty() {
            continue;
        }
        let result_status = if status == "ok" {
            Status::Success
        } else {
            Status::Failed
        };
        match deploy_task::update_pending(
            task.ip.clone(),
            task_id.to_string(),
            result_status.clone(),
            String::new(),
        )
        .await
        {
            Ok(_) => info!(task_id = task_id, ip = task.ip, "Update task success",),
            Err(e) => warn!(
                task_id = task_id,
                ip = task.ip,
                "Update task failed: {:?}",
                e,
            ),
        }
    }
}
