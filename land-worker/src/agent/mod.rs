use self::conf::handle_task;
use anyhow::{anyhow, Result};
use land_common::IPInfo;
use once_cell::sync::Lazy;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, warn};

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

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            if let Err(e) = run_inner(addr.clone(), token.clone()).await {
                warn!("Run agent background failed: {}", e);
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
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&req)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        warn!("Bad response status: {}, body: {}", status, text);
        return Err(anyhow!("Bad response status: {}", status));
    }
    let values: Vec<String> = resp.json().await?;
    // if key in results is not in values, remove key
    results.retain(|k, _| values.contains(k));
    for value in values {
        tokio::spawn(async move {
            let task: TaskValue = serde_json::from_str(value.as_str()).unwrap();
            let mut results = TASKS_RESULT.lock().await;
            if results.contains_key(&task.task_id) {
                info!("Task already handled: {}", task.task_id);
                return;
            }
            match handle_task(&task).await {
                Ok(_) => {
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
struct TaskValue {
    user_uuid: String,
    project_uuid: String,
    domain: String,
    download_url: String,
    wasm_path: String,
    task_id: String,
}
