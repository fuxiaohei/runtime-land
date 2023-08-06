use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RouteConfItem is config item for one project deployment route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfItem {
    pub domain: String,
    pub module: String,
    pub key: String,
    pub time_at: u64,
    pub md5: String,
}

impl RouteConfItem {
    pub fn new(domain: String, module: String, key: String, time_at: u64) -> Self {
        let mut hasher = Md5::new();
        hasher.update(format!("{}-{}-{}", domain, module, key));
        let result = hasher.finalize();
        let md5 = format!("{:x}", result);
        Self {
            domain,
            module,
            key,
            time_at,
            md5,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegionIPInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub readme: String,
}

impl RegionIPInfo {
    pub fn region(&self) -> String {
        format!("{}-{}-{}", self.country, self.region, self.city)
    }
    pub fn region_ip(&self) -> String {
        format!("{}-{}-{}-{}", self.country, self.region, self.city, self.ip)
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
    pub localip: RegionIPInfo,
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
}
