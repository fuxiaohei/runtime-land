use anyhow::Result;
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Once;
use tracing::info;

mod confs;
mod livings;
mod sync;
mod task;
mod traefik;

pub use confs::{get_confs, init_confs, Item};
pub use livings::{init_livings, set_living};
pub use sync::init_sync;
pub use task::init_task;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct IP {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub hostname: Option<String>,
}

const IPINFO_LINK: &str = "https://ipinfo.io/json";

/// IPDATA is global once cell for ipinfo data
static IPDATA: OnceCell<IP> = OnceCell::new();

/// init_ip gets ip info from ipinfo.io
pub async fn init_ip(ip: Option<String>) -> Result<()> {
    if let Some(ip) = ip {
        IPDATA
            .set(IP {
                ip,
                ..Default::default()
            })
            .unwrap();
        return Ok(());
    }
    let resp = reqwest::get(IPINFO_LINK).await?;
    let mut ip_info: IP = resp.json().await?;
    ip_info.hostname = Some(land_common::get_hostname()?);
    info!("IP info: {:?}", ip_info);
    IPDATA.set(ip_info).unwrap();
    Ok(())
}
/// get gets ip info from global variable
pub async fn get_ip() -> IP {
    let ip_data = IPDATA.get().unwrap();
    ip_data.clone()
}

static CLIENT: OnceCell<Client> = OnceCell::new();
static CLIENT_ONCE: Once = Once::new();
