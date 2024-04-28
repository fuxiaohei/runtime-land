use self::conf::handle_task;
use crate::agent::conf::handle_total;
use anyhow::{anyhow, Result};
use land_common::IPInfo;
use land_dao::confs::TaskValue;
use once_cell::sync::Lazy;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

mod conf;
pub mod ip;

/// DATA_DIR is the directory to store data
static DATA_DIR: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::from("data")));

/// TASKS_RESULT is the global variable to store task results
pub static TASKS_RESULT: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// run_background runs the agent background tasks
pub async fn run_background(addr: String, token: String, dir: String) {
    let mut data_dir = DATA_DIR.lock().await;
    *data_dir = dir;

    // sync current tasks to deploy from cloud-server for every second
    let addr2 = addr.clone();
    let token2 = token.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            if let Err(e) = run_inner(addr2.clone(), token2.clone()).await {
                warn!("Run agent background failed: {}", e);
            }
        }
    });

    // check all deploys changes from cloud-server every minute
    // FIXME: it should check all deploys changes before worker starts serve
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(e) = run_all(addr.clone(), token.clone()).await {
                warn!("Run all deploys failed: {}", e);
            }
        }
    });
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    pub ip: IPInfo,
    pub tasks: HashMap<String, String>,
}

// global reqwest client to reuse
static CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

async fn run_inner(addr: String, token: String) -> Result<()> {
    let url = format!("{}/api/v1/worker-api/alive", addr);
    let mut results = TASKS_RESULT.lock().await;
    let req = Request {
        ip: ip::get().await,
        tasks: results.clone(),
    };
    let resp = CLIENT
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&req)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        warn!("Bad response status: {}, body: {}", status, text);
        return Err(anyhow!("Bad response status: {}, url: {}", status, url));
    }
    let values: Vec<String> = resp.json().await?;
    // if key in results is not in values, remove key
    results.retain(|k, _| values.contains(k));
    if values.is_empty() {
        // debug!("No tasks to handle");
        return Ok(());
    }
    for value in values {
        tokio::spawn(async move {
            let task: TaskValue = serde_json::from_str(value.as_str()).unwrap();
            let mut results = TASKS_RESULT.lock().await;
            if results.contains_key(&task.task_id) {
                info!("Task already handled: {}", task.task_id);
                return;
            }
            debug!(task_id = task.task_id, "Handle task");
            match handle_task(&task).await {
                Ok(_) => {
                    info!(task_id = task.task_id, "Handle task success");
                    results.insert(task.task_id.clone(), "success".to_string());
                }
                Err(e) => {
                    results.insert(task.task_id.clone(), format!("failed: {}", e));
                    warn!("Handle task failed: {}", e);
                }
            }
        });
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskTotal {
    checksum: String,
    tasks: Vec<TaskValue>,
}

/// run_all sync all deploys data to local and compare changes
async fn run_all(addr: String, token: String) -> Result<()> {
    let url = format!("{}/api/v1/worker-api/deploys", addr);
    let resp = CLIENT
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        warn!("Bad response status: {}, body: {}", status, text);
        return Err(anyhow!("Bad response status: {}", status));
    }
    let total = resp.json::<TaskTotal>().await?;
    compare_total(&total).await?;
    Ok(())
}

async fn compare_total(total: &TaskTotal) -> Result<()> {
    let data_dir = DATA_DIR.lock().await;
    let local_file = format!("{}/deploys.json", data_dir);
    // if local file not exists, write total to local file
    if !std::path::Path::new(&local_file).exists() {
        handle_total(data_dir.as_str(), total).await?;
        let content = serde_json::to_string(total)?;
        std::fs::write(&local_file, content)?;
        info!("Write deploys to local file: {}", local_file);
        return Ok(());
    }

    // read local file and compare with total
    let old_bytes = std::fs::read(&local_file)?;
    let local_total = serde_json::from_slice::<TaskTotal>(&old_bytes)?;
    if local_total.checksum == total.checksum {
        debug!("No changes in deploys");
        return Ok(());
    }

    // write new total to local file
    handle_total(data_dir.as_str(), total).await?;
    let content = serde_json::to_string(total)?;
    std::fs::write(&local_file, content)?;
    Ok(())
}
