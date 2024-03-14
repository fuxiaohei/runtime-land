use anyhow::Result;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

use crate::gateway::ConfData;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub ip: crate::ip::IpInfo,
    pub checksum: String,
    pub deploys: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub data: ConfData,
}

pub struct Opts {
    pub cloud_server_addr: String,
    pub token: String,
    pub data_dir: String,
    pub conf_file: String,
    pub server_addr: String,
}

/// run_loop will run the sync loop every `interval` seconds
pub fn run_loop(interval: i32, opt: Opts) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval as u64));
        let url = format!("{}/api/worker/v1/deploys", opt.cloud_server_addr);
        loop {
            ticker.tick().await;
            match run(&url, &opt).await {
                Ok(_) => {}
                Err(e) => warn!("Loop error: {:?}", e),
            }
        }
    });
}

#[instrument("[sync]", skip_all)]
pub async fn run(url: &str, opt: &Opts) -> Result<()> {
    let checksum = crate::gateway::DATA.lock().await.checksum.clone();
    let deploys = super::DEPLOY_RES.lock().await.clone();
    let req = SyncRequest {
        ip: crate::ip::IP_DATA.lock().await.clone(),
        checksum: checksum.clone(),
        deploys,
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", opt.token))
        .json(&req)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        // if not modified, just return
        if status == reqwest::StatusCode::NOT_MODIFIED {
            debug!("Not change: {}", checksum);
            return Ok(());
        }
        warn!("Bad status code: {}", status);
        return Err(anyhow::anyhow!("Bad status code: {}", status));
    }

    let resp: SyncResponse = resp.json().await?;
    debug!("Resp: {}", resp.data.checksum);
    let mut data = crate::gateway::DATA.lock().await;
    *data = resp.data;

    // write file once
    let contents = super::handle_data(&data, opt).await?;
    std::fs::write(&opt.conf_file, contents)?;

    Ok(())
}
