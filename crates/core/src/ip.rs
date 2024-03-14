use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::info;

const IPINFO_LINK: &str = "https://ipinfo.io/json";

/*
{
    "ip": "27.148.194.74",
    "city": "Xiamen",
    "region": "Fujian",
    "country": "CN",
    "loc": "24.4798,118.0819",
    "org": "AS133775 Xiamen",
    "timezone": "Asia/Shanghai",
    "readme": "https://ipinfo.io/missingauth"
  }
   */

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub hostname: Option<String>,
}

/// init gets ip info from ipinfo.io
pub async fn init() -> Result<()> {
    let resp = reqwest::get(IPINFO_LINK).await?;
    let mut ip_info: IpInfo = resp.json().await?;
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
pub async fn get() -> IpInfo {
    let ip_data = IP_DATA.lock().await;
    ip_data.clone()
}

/// IP_DATA is a global variable to store ip info
pub static IP_DATA: Lazy<Mutex<IpInfo>> = Lazy::new(|| {
    Mutex::new(IpInfo {
        ip: "127.0.0.1".to_string(),
        city: "".to_string(),
        region: "".to_string(),
        country: "".to_string(),
        loc: "".to_string(),
        org: "".to_string(),
        timezone: "".to_string(),
        hostname: Some("".to_string()),
    })
});
