use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// DeployRes is a type to store deployment result
pub type DeployRes = HashMap<String, String>;

/// DEPLOY_RES is a global variable to store deployment result
pub static DEPLOY_RES: Lazy<Mutex<DeployRes>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// get_res returns the global deployment result
pub async fn get_res() -> DeployRes {
    DEPLOY_RES.lock().await.clone()
}

pub async fn build(dir: String) {
    let data = super::sync::get().await;
    let mut deploys = DEPLOY_RES.lock().await;
    for item in data.items {
        if item.status == "success" {
            // if status is success, remove from deploys
            if deploys.contains_key(&item.key) {
                info!("Remove by success:{}", item.key);
                deploys.remove(&item.key);
            }
            continue;
        }
        let wasm_file = format!("{}/{}", dir, item.path);
        // if wasm_file not exist, download from item.down_url
        if !std::path::Path::new(&wasm_file).exists() {
            match download_item(item.dl_url, wasm_file).await {
                Ok(_) => {
                    info!("Downloaded: {}", item.key);
                    deploys.insert(item.key, "ok".to_string());
                }
                Err(e) => {
                    info!("Download error:{:?}", e);
                    deploys.insert(item.key, format!("error:{}", e));
                }
            }
        } else {
            debug!("Already exist:{}", item.key);
            deploys.insert(item.key, "ok".to_string());
        }
    }
}

async fn download_item(url: String, target: String) -> Result<()> {
    let resp = reqwest::get(&url).await?;
    let bytes = resp.bytes().await?;
    let bytes_len = bytes.len();
    let dir = std::path::Path::new(&target).parent().unwrap();
    std::fs::create_dir_all(dir)?;
    std::fs::write(&target, bytes)?;
    info!("Downloaded:{}, size:{}", target, bytes_len);
    Ok(())
}
