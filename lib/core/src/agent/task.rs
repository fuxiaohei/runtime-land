use super::Item;
use anyhow::{anyhow, Result};
use land_dao::deploy_task::TaskType;
use land_vars::Task;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use tokio::sync::Mutex;
use tracing::{debug, instrument, warn};

#[derive(Deserialize, Default, Clone, Debug)]
struct SyncResponse {
    pub status: String,
    pub message: String,
    pub data: Vec<Task>,
}

/// init_task starts background tasks
pub async fn init_task(addr: String, token: String, dir: String) {
    debug!("agent init_task");

    // init client
    super::CLIENT_ONCE.call_once(|| {
        let client = Client::new();
        super::CLIENT.set(client).unwrap();
    });

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match request(addr.clone(), token.clone(), dir.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent task error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

lazy_static! {
    static ref TASK_RES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[instrument("[AGT-TASK]", skip_all)]
async fn request(addr: String, token: String, dir: String) -> Result<()> {
    let ipinfo = super::get_ip().await;
    let client = super::CLIENT.get().unwrap();
    let mut tasks = TASK_RES.lock().await;

    let api = format!("{}/worker-api/task?ip={}", addr, ipinfo.ip);
    let token = format!("Bearer {}", token);
    let res = client
        .post(api)
        .header("Authorization", token)
        .header("X-Md5", "".to_string())
        .json(&tasks.clone())
        .send()
        .await?;

    let status_code = res.status().as_u16();
    if status_code == 204 {
        tasks.clear();
        // debug!("no change");
        return Ok(());
    }
    // 400+ is error
    if status_code >= 400 {
        let content = res.text().await?;
        return Err(anyhow!("Bad status:{}, Error:{}", status_code, content));
    }
    let resp: SyncResponse = res.json().await?;
    if resp.status != "ok" {
        warn!("Bad response: {}", resp.message);
        return Err(anyhow!("Bad response: {}", resp.message));
    }
    // debug!("sync response: {}, {}", resp.status, resp.message);
    if resp.data.is_empty() {
        tasks.clear();
        // debug!("no task");
        return Ok(());
    }
    debug!("sync task: {:?}", resp.data);

    // remove not exist task in task-res from current task response
    let current_task_keys = resp
        .data
        .iter()
        .map(|t| t.task_id.clone())
        .collect::<Vec<String>>();
    tasks.retain(|k, _| current_task_keys.contains(k));

    // handle each task
    for task in resp.data {
        let task_id = task.task_id.clone();
        match handle_each_task(task, dir.clone()).await {
            Ok(_) => {
                tasks.insert(task_id, "success".to_string());
            }
            Err(e) => {
                warn!(task_id = task_id, "handle task error: {:?}", e);
                tasks.insert(task_id, e.to_string());
            }
        }
    }

    Ok(())
}

async fn handle_each_task(t: Task, dir: String) -> Result<()> {
    if t.task_type == TaskType::DeployWasmToWorker.to_string() {
        let item: Item = serde_json::from_str(&t.content)?;
        handle_each_agent_item(item, dir.clone()).await?;
        return Ok(());
    }
    Err(anyhow!("unknown task type: {}", t.task_type))
}

async fn handle_each_agent_item(item: Item, dir: String) -> Result<()> {
    println!("handle each item: {:?}", item);
    let wasm_target_file = format!("{}/{}", dir, item.file_name);

    // 1. download wasm file
    if !Path::new(&wasm_target_file).exists() {
        let resp = reqwest::get(&item.download_url).await?;
        if resp.status().as_u16() != 200 {
            return Err(anyhow!(
                "download error: {}, url: {}",
                resp.status(),
                item.download_url
            ));
        }
        let content = resp.bytes().await?;
        let content_md5 = format!("{:x}", md5::compute(&content));
        if content_md5 != item.file_hash {
            return Err(anyhow!(
                "download hash dismatch: real: {}, expect: {}, url: {}",
                content_md5,
                item.file_hash,
                item.download_url,
            ));
        }
        let dir = Path::new(&wasm_target_file).parent().unwrap();
        std::fs::create_dir_all(dir)?;
        std::fs::write(&wasm_target_file, content)?;
        debug!("download success: {}", wasm_target_file);
    }

    // 2. generate traefic file
    let traefik_file = format!("{}/traefik/{}.yaml", dir, item.domain.replace('.', "_"));
    let traefik_dir = format!("{}/traefik", dir);
    std::fs::create_dir_all(traefik_dir)?;
    let confs = super::traefik::build(&item, "land-worker")?;
    let content = serde_yaml::to_string(&confs)?;
    std::fs::write(&traefik_file, content)?;
    debug!("generate traefik success: {}", traefik_file);

    Ok(())
}
