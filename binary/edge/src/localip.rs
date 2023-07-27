use std::sync::OnceLock;

use anyhow::Result;
use tracing::{debug, info, instrument};

const IPINFO_LINK: &str = "https://ipinfo.io/json";
const IPINFO_LOCAL_FILE: &str = "ipinfo.json";
pub static IPINFO: OnceLock<IpInfo> = OnceLock::new();

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

#[derive(serde::Deserialize, Clone, serde::Serialize, Debug)]
pub struct IpInfo {
    ip: String,
    city: String,
    region: String,
    country: String,
    loc: String,
    org: String,
    timezone: String,
    readme: String,
}

impl IpInfo {
    pub fn region(&self) -> String {
        format!("{}-{}-{}", self.country, self.region, self.city)
    }
    pub fn region_ip(&self) -> String {
        format!("{}-{}-{}-{}", self.country, self.region, self.city, self.ip)
    }
}

#[instrument(name = "[LocalIP]")]
pub async fn init() -> Result<()> {
    let ip = match read_file() {
        Ok(ip) => ip,
        Err(_) => {
            let ip = reqwest::get(IPINFO_LINK).await?.json::<IpInfo>().await?;
            debug!("remoteip: {:?}", ip);
            std::fs::write(IPINFO_LOCAL_FILE, serde_json::to_string(&ip)?)?;
            ip
        }
    };
    info!("ip : {:?}, region: {}", ip, ip.region());
    IPINFO.get_or_init(|| ip);
    Ok(())
}

fn read_file() -> Result<IpInfo> {
    let file = std::fs::File::open(IPINFO_LOCAL_FILE)?;
    let reader = std::io::BufReader::new(file);
    let ip = serde_json::from_reader(reader)?;
    Ok(ip)
}
