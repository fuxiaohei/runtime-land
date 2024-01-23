use anyhow::Result;
use serde::{Deserialize, Serialize};

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
}

/// get_ip_info get ip info from ipinfo.io
pub fn get_ip_info() -> Result<IpInfo> {
    let resp = ureq::get(IPINFO_LINK).call()?;
    let ip_info: IpInfo = resp.into_json()?;
    Ok(ip_info)
}
