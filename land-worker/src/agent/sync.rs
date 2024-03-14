use anyhow::{anyhow, Result};
use land_common::ip;
use land_kernel::cron::ConfData;
use once_cell::sync::Lazy;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub ip: ip::Info,
    pub checksum: String,
    pub deploys: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub data: ConfData,
}

/// global data
static DATA: Lazy<Mutex<ConfData>> = Lazy::new(|| {
    Mutex::new(ConfData {
        items: vec![],
        checksum: "".to_string(),
    })
});

/// get returns the global data
pub async fn get() -> ConfData {
    let data = DATA.lock().await;
    data.clone()
}

/// start starts the sync loop
pub async fn start(interval: u64, addr: String, token: String, dir: String) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval));
        loop {
            ticker.tick().await;
            match sync(addr.clone(), token.clone(), dir.clone()).await {
                Ok(_) => {}
                Err(e) => tracing::warn!("Loop error: {:?}", e),
            }
        }
    });
}

#[instrument("[SYNC]", skip_all)]
async fn sync(addr: String, token: String, dir: String) -> Result<()> {
    let url = format!("{}/api/worker/v1/deploys", addr);
    let mut data = DATA.lock().await;
    let req = Request {
        ip: super::ip::get().await,
        checksum: data.checksum.clone(),
        deploys: super::traefik::get_res().await,
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&req)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        // if not modified, just return
        if status == reqwest::StatusCode::NOT_MODIFIED {
            debug!("Not change, checksum: {}", data.checksum);
            return Ok(());
        }
        warn!("Bad status code: {}", status);
        return Err(anyhow!("Bad status code: {}", status));
    }
    let value: Response = resp.json().await?;
    *data = value.data;
    info!("Sync success, checksum: {}", data.checksum);

    tokio::spawn(async move {
        super::traefik::build(dir).await;
    });

    Ok(())
}
