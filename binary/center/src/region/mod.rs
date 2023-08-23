use land_core::confdata::RegionReportData;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{warn, Instrument};

mod recv;
mod refresh;
pub use recv::build_data as build_recv_data;

pub mod conf;

lazy_static! {
    pub static ref REGIONS: Mutex<HashMap<String, RegionReportData>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

/// REGION_REFRESH_INTERVAL is the interval to refresh REGIONS to database
const REGION_REFRESH_INTERVAL: u64 = 30;
/// REGION_INACTIVE_EXPIRE is the expiry to check if region is inactive
const REGION_INACTIVE_EXPIRE: u64 = 120;

pub async fn init() {
    // start 10s interval to update REGIONS to database
    tokio::spawn(
        async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(REGION_REFRESH_INTERVAL));
            loop {
                interval.tick().await;
                match refresh::refresh().await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("refresh regions error: {:?}", e);
                    }
                }
            }
        }
        .instrument(tracing::info_span!("[REGION]")),
    );

    conf::init().await;
}
