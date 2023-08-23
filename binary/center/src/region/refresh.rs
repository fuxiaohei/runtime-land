use anyhow::Result;
use land_core::confdata::RegionReportData;
use tracing::{info, warn};

fn report_data_to_model(rg: &RegionReportData, key: String) -> land_dao::Region {
    let now = chrono::Utc::now();
    land_dao::Region {
        id: 0,
        name: rg.localip.region(),
        key,
        ip: rg.localip.ip.clone(),
        city: rg.localip.city.clone(),
        country: rg.localip.country.clone(),
        runtimes: rg.runtimes.len() as i32,
        owner_id: rg.owner_id,
        status: land_dao::region::Status::Active.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    }
}

pub async fn refresh() -> Result<()> {
    // get regions from database
    let saved_regions = land_dao::region::list_maps().await?;

    // get active regions from REGIONS
    let mut regions = super::REGIONS.lock().await;
    let now_ts = chrono::Utc::now().timestamp() as u64;

    // compare saved_regions and regions
    // iterate regions. if region not in saved_regions, create it to database
    let mut expired_keys = vec![];
    for (key, region_data) in regions.iter_mut() {
        let expired = now_ts - region_data.time_at > super::REGION_INACTIVE_EXPIRE;
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
        let model = report_data_to_model(region_data, key.clone());
        land_dao::region::create(model).await?;
    }

    // iterate saved_regions. if region not in regions, set it inactive
    for (key, region) in saved_regions.iter() {
        // region record is handled by REGIONS
        if regions.contains_key(key) {
            continue;
        }
        // region record is not expired
        if region.updated_at.timestamp() as u64 + super::REGION_INACTIVE_EXPIRE > now_ts {
            continue;
        }
        // region record is already inactive
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
