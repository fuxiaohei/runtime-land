use crate::gateway::traefik::build_data_yaml;
use crate::gateway::ConfData;
use crate::workerinfo::sync::Opts;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::info;

pub mod sync;

/// WorkerDeployRes is a type to store deployment result
pub type WorkerDeployRes = HashMap<String, String>;

/// DEPLOY_RES is a global variable to store deployment result
pub static DEPLOY_RES: Lazy<Mutex<WorkerDeployRes>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// handle_data will handle deploy datas, build traefik configuration and convert it to yaml
pub async fn handle_data(data: &ConfData, opt: &Opts) -> Result<String> {
    let mut dp_res = DEPLOY_RES.lock().await;
    // check and download wasm file
    for item in data.items.iter() {
        // deploying status should handle item
        if item.status == "deploying" {
            let wasm_file = format!("{}/{}", opt.data_dir, item.path);
            // if wasm_file not exist, download file to wasm_file
            if !std::path::Path::new(&wasm_file).exists() {
                let resp = reqwest::get(&item.dl_url).await?;
                let bytes = resp.bytes().await?;
                let bytes_len = bytes.len();
                let wasm_dir = std::path::Path::new(&wasm_file).parent().unwrap();
                std::fs::create_dir_all(wasm_dir)?;
                std::fs::write(&wasm_file, bytes)?;
                info!("Downloaded:{}, size:{}", wasm_file, bytes_len);
            }
            dp_res.insert(item.key.clone(), "ok".to_string());
            continue;
        }
        if item.status == "success" {
            // if status is success, remove from dp_res
            info!("Remove by success:{}", item.key);
            dp_res.remove(&item.key);
        }
    }
    // if dp_res key not exist in data, remove it
    let mut removes = vec![];
    for key in dp_res.keys() {
        if !data.items.iter().any(|item| item.key == *key) {
            removes.push(key.clone());
        }
    }
    if !removes.is_empty() {
        for key in removes {
            info!("Remove by not exist:{}", key);
            dp_res.remove(&key);
        }
    }
    build_data_yaml(data, &opt.server_addr).await
}
