use anyhow::{anyhow, Result};
use reqwest::Client;
use tracing::{debug, instrument, warn};

#[instrument("[AGT-SYNC]", skip_all)]
async fn request(addr: String, token: String) -> Result<()> {
    let ipinfo = super::get_ip().await;
    let client = super::CLIENT.get().unwrap();

    let api = format!("{}/worker-api/sync", addr);
    println!("api:{}", api);
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
    Ok(())
}

/// init_background starts background tasks
pub async fn init_sync(addr: String, token: String) {
    debug!("agent init_sync");

    // init client
    let client = Client::new();
    super::CLIENT.set(client).unwrap();

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match request(addr.clone(), token.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent ping error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}
