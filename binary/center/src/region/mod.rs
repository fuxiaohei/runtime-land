use anyhow::Result;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, warn, Instrument};

mod ws;
pub use ws::ws_handler;

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

impl IpInfo {
    pub fn region(&self) -> String {
        format!("{}-{}-{}", self.country, self.region, self.city)
    }
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
    #[serde(skip)]
    pub owner_id: i32,
    #[serde(skip)]
    pub time_at: u64,
}

impl RegionData {
    pub fn to_model(&self, key: String) -> land_dao::Region {
        let now = chrono::Utc::now();
        land_dao::Region {
            id: 0,
            name: self.localip.region(),
            key,
            ip: self.localip.ip.clone(),
            city: self.localip.city.clone(),
            country: self.localip.country.clone(),
            runtimes: self.runtimes.len() as i32,
            owner_id: self.owner_id,
            status: land_dao::region::Status::Active.to_string(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

lazy_static! {
    pub static ref REGIONS: Mutex<HashMap<String, RegionData>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

/// REGION_REFRESH_INTERVAL is the interval to refresh REGIONS to database
const REGION_REFRESH_INTERVAL: u64 = 10;
/// REGION_INACTIVE_EXPIRE is the expiry to check if region is inactive
const REGION_INACTIVE_EXPIRE: u64 = 60;

pub async fn init() {
    // start 10s interval to update REGIONS to database
    tokio::spawn(
        async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(REGION_REFRESH_INTERVAL));
            loop {
                interval.tick().await;
                match refresh_regions().await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("refresh regions error: {:?}", e);
                    }
                }
            }
        }
        .instrument(tracing::info_span!("[REGION]")),
    );
}

async fn refresh_regions() -> Result<()> {
    // get regions from database
    let saved_regions = land_dao::region::list_maps().await?;

    // get active regions from REGIONS
    let mut regions = REGIONS.lock().await;
    let now_ts = chrono::Utc::now().timestamp() as u64;

    // compare saved_regions and regions
    // iterate regions. if region not in saved_regions, create it to database
    let mut expired_keys = vec![];
    for (key, region_data) in regions.iter_mut() {
        let expired = now_ts - region_data.time_at > REGION_INACTIVE_EXPIRE;
        if expired {
            expired_keys.push(key.clone());
        }
        if saved_regions.contains_key(key) {
            if expired {
                warn!("{} expired and set inactive", key);
                land_dao::region::set_inactive(key.clone()).await?;
                continue;
            }

            land_dao::region::update_runtimes(key.clone(), region_data.runtimes.len() as i32)
                .await?;
            info!("updated {}, runtimes: {}", key, region_data.runtimes.len());
            continue;
        }
        info!("create {:?}: {:?}", key, region_data);
        let model = region_data.to_model(key.clone());
        land_dao::region::create(model).await?;
    }

    // iterate saved_regions. if region not in regions, set it inactive
    for (key, region) in saved_regions.iter() {
        if regions.contains_key(key) {
            continue;
        }
        if region.status == land_dao::region::Status::InActive.to_string() {
            continue;
        }
        warn!("set {} inactive", key);
        land_dao::region::set_inactive(key.clone()).await?;
    }

    // remove expired regions
    for key in expired_keys {
        regions.remove(&key);
    }

    Ok(())
}
