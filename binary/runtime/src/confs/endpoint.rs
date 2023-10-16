use anyhow::Result;
use land_core::confdata::RuntimeNodeInfo;
use once_cell::sync::OnceCell;
use tracing::{debug, info, instrument};

/// ENDPOINT is the name of endpoint
pub static ENDPOINT: OnceCell<String> = OnceCell::new();
/// ENDPOINT_INFO is the info of endpoint
pub static ENDPOINT_INFO: OnceCell<RuntimeNodeInfo> = OnceCell::new();

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
    let info = match read_file() {
        Ok(ip) => ip,
        Err(_) => {
            let ip = reqwest::get(IPINFO_LINK)
                .await?
                .json::<RuntimeNodeInfo>()
                .await?;
            debug!("remoteip: {:?}", ip);
            std::fs::write(IPINFO_LOCAL_FILE, serde_json::to_string(&ip)?)?;
            ip
        }
    };
    info!("ip : {:?}, region: {}", info, info.region());
    ENDPOINT.get_or_init(|| info.region_hostname(hostname.to_str().unwrap()));
    ENDPOINT_INFO.get_or_init(|| info);
    Ok(())
}

fn read_file() -> Result<RuntimeNodeInfo> {
    let file = std::fs::File::open(IPINFO_LOCAL_FILE)?;
    let reader = std::io::BufReader::new(file);
    let ip = serde_json::from_reader(reader)?;
    Ok(ip)
}
