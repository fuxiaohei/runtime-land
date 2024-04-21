use anyhow::Result;
use land_common::IPInfo;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tracing::info;

const IPINFO_LINK: &str = "https://ipinfo.io/json";

/// IP_DATA is a global variable to store ip info
static IP_DATA: Lazy<Mutex<IPInfo>> = Lazy::new(|| {
    Mutex::new(IPInfo {
        ip: "127.0.0.1".to_string(),
        city: "".to_string(),
        region: "".to_string(),
        country: "".to_string(),
        loc: "".to_string(),
        org: "".to_string(),
        timezone: "".to_string(),
        hostname: Some("localhost".to_string()),
    })
});

/// init gets ip info from ipinfo.io
pub async fn init() -> Result<()> {
    let resp = reqwest::get(IPINFO_LINK).await?;
    let mut ip_info: IPInfo = resp.json().await?;
    ip_info.hostname = Some(
        hostname::get()
            .unwrap_or("unknown".into())
            .to_string_lossy()
            .to_string(),
    );
    info!("IP info: {:?}", ip_info);
    let mut ip_data = IP_DATA.lock().await;
    *ip_data = ip_info;
    Ok(())
}

/// get gets ip info from global variable
pub async fn get() -> IPInfo {
    let ip_data = IP_DATA.lock().await;
    ip_data.clone()
}
