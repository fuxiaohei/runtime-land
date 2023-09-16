use anyhow::Result;
use land_core::confdata::RegionIPInfo;
use once_cell::sync::OnceCell;
use tracing::{debug, info, instrument};

/// ENDPOINT is the name of endpoint
pub static ENDPOINT: OnceCell<String> = OnceCell::new();

const IPINFO_LINK: &str = "https://ipinfo.io/json";
const IPINFO_LOCAL_FILE: &str = "ipinfo.json";

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

#[instrument(name = "[LocalIP]")]
pub async fn init() -> Result<()> {
    let hostname = gethostname::gethostname();
    let ip = match read_file() {
        Ok(ip) => ip,
        Err(_) => {
            let ip = reqwest::get(IPINFO_LINK)
                .await?
                .json::<RegionIPInfo>()
                .await?;
            debug!("remoteip: {:?}", ip);
            std::fs::write(IPINFO_LOCAL_FILE, serde_json::to_string(&ip)?)?;
            ip
        }
    };
    info!("ip : {:?}, region: {}", ip, ip.region());
    ENDPOINT.get_or_init(|| ip.region_ip_hostname(hostname.to_str().unwrap()));
    Ok(())
}

fn read_file() -> Result<RegionIPInfo> {
    let file = std::fs::File::open(IPINFO_LOCAL_FILE)?;
    let reader = std::io::BufReader::new(file);
    let ip = serde_json::from_reader(reader)?;
    Ok(ip)
}
