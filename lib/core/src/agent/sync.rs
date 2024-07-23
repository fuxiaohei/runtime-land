use super::Item;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, instrument, warn};

#[derive(Deserialize, Default, Clone, Debug)]
struct SyncResponse {
    pub status: String,
    pub message: String,
    pub data: Vec<Item>,
}

#[instrument("[AGT-SYNC]", skip_all)]
async fn request(addr: String, token: String, dir: String) -> Result<()> {
    let ipinfo = super::get_ip().await;
    let client = super::CLIENT.get().unwrap();

    let api = format!("{}/worker-api/sync", addr);
    let token = format!("Bearer {}", token);
    let res = client
        .post(api)
        .header("Authorization", token)
        .header("X-Md5", "".to_string())
        .json(&ipinfo)
        .send()
        .await?;

    let status_code = res.status().as_u16();
    if status_code == 304 {
        // debug!("no change");
        return Ok(());
    }
    // 400+ is error
    if status_code >= 400 {
        let content = res.text().await?;
        return Err(anyhow!("Bad status:{}, Error:{}", status_code, content));
    }
    let resp: SyncResponse = res.json().await?;
    let conf_file = format!("{}/confs.json", dir);
    if resp.status != "ok"{
        return Err(anyhow!("sync error: {}", resp.message));
    }
    // debug!("sync data: {}, {}", resp.status, resp.message);
    // write resp to file
    std::fs::write(conf_file, serde_json::to_string(&resp.data).unwrap()).unwrap();
    Ok(())
}

/// init_background starts background tasks
pub async fn init_sync(addr: String, token: String, dir: String) {
    debug!("agent init_sync");

    // init client
    super::CLIENT_ONCE.call_once(|| {
        let client = Client::new();
        super::CLIENT.set(client).unwrap();
    });

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));
        ticker.tick().await;
        loop {
            match request(addr.clone(), token.clone(), dir.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent ping error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}
