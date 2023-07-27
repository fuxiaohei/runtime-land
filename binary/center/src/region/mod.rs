use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct IpInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub readme: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeData {
    pub hostname: String,
    pub cpu_count: usize,
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub updated_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct RegionData {
    pub localip: IpInfo,
    pub region: String,
    pub runtimes: HashMap<String, RuntimeData>,
}

lazy_static! {
    pub static ref REGIONS: Mutex<HashMap<String, RegionData>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}
