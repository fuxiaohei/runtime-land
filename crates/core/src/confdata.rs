use md5::{Digest, Md5};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;

/// RouteConfItem is config item for one project deployment route
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RouteConfItem {
    pub domain: String,
    pub module: String,
    pub key: String,
    pub time_at: u64,
    pub md5: String,
    pub download_url: String,
}

impl RouteConfItem {
    pub fn new(
        domain: String,
        module: String,
        key: String,
        download_url: String,
        time_at: u64,
    ) -> Self {
        let mut hasher = Md5::new();
        hasher.update(format!("{}|{}|{}|{}", domain, module, key, download_url));
        let result = hasher.finalize();
        let md5 = format!("{:x}", result);
        Self {
            domain,
            module,
            key,
            time_at,
            md5,
            download_url,
        }
    }
}

// RoutesConf is config for all project deployment routes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutesConf {
    pub items: Vec<RouteConfItem>,
    pub created_at: u64,
}

impl RoutesConf {
    pub fn to_map(&self) -> HashMap<String, RouteConfItem> {
        let mut map = HashMap::new();
        for item in &self.items {
            map.insert(item.key.clone(), item.clone());
        }
        map
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EndpointConf {
    pub items: Vec<RouteConfItem>,
    pub created_at: u64,
    pub md5: String,
}

impl EndpointConf {
    pub fn to_map(&self) -> HashMap<String, RouteConfItem> {
        let mut map = HashMap::new();
        for item in &self.items {
            map.insert(item.key.clone(), item.clone());
        }
        map
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeNodeInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub readme: String,
    pub conf_hash: Option<String>,
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

impl RuntimeNodeInfo {
    pub fn region(&self) -> String {
        let mut s = format!("{}-{}-{}", self.country, self.region, self.city);
        remove_whitespace(&mut s);
        s
    }
    pub fn region_ip(&self) -> String {
        let mut s = format!("{}-{}-{}-{}", self.country, self.region, self.city, self.ip);
        remove_whitespace(&mut s);
        s
    }
    pub fn region_hostname(&self, hostname: &str) -> String {
        let mut s = format!(
            "{}-{}-{}-{}",
            self.country, self.region, self.city, hostname
        );
        remove_whitespace(&mut s);
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeData {
    pub hostname: String,
    pub cpu_count: usize,
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionReportData {
    pub localip: RuntimeNodeInfo,
    pub region: String,
    pub runtimes: HashMap<String, RuntimeData>,
    pub conf_value_time_version: u64,
    pub time_at: u64,

    #[serde(skip)]
    pub owner_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionRecvData {
    pub conf_values: RoutesConf,
    pub storage_basepath: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeRecvData {
    pub region_name: String,
}

#[derive(Debug)]
/// DomainSetting is the domain settings
pub struct DomainSetting {
    pub domain: String,
    pub protocol: String,
}

static DOMAIN_SETTING: Lazy<Mutex<DomainSetting>> = Lazy::new(|| {
    Mutex::new(DomainSetting {
        domain: "".to_string(),
        protocol: "".to_string(),
    })
});

/// set_domain sets the domain to access the function
pub async fn set_domain(domain: String, protocol: String) {
    let mut d = DOMAIN_SETTING.lock().await;
    d.domain = domain;
    d.protocol = protocol;
}

/// get_domain gets the domain to access the function
pub async fn get_domain() -> (String, String) {
    let d = DOMAIN_SETTING.lock().await;
    (d.domain.clone(), d.protocol.clone())
}

#[cfg(test)]
mod tests {
    use super::RuntimeNodeInfo;

    #[test]
    fn region_ipinfo() {
        let r = RuntimeNodeInfo {
            ip: "1.1.1.1".to_string(),
            city: "Los Angeles".to_string(),
            region: "California".to_string(),
            country: "US".to_string(),
            loc: "34.0544,-118.2440".to_string(),
            org: "AS13335 Cloudflare, Inc.".to_string(),
            timezone: "America/Los_Angeles".to_string(),
            readme: "https://ipinfo.io/missingauth".to_string(),
            conf_hash: None,
        };
        let region_str = r.region();
        let region_ip_str = r.region_ip();
        assert_eq!(region_str, "US-California-LosAngeles");
        assert_eq!(region_ip_str, "US-California-LosAngeles-1.1.1.1");
    }
}
