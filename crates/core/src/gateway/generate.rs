use super::{ConfData, ConfItem, DATA};
use anyhow::Result;
use tracing::{debug, info, instrument};

async fn is_should_generate() -> Result<bool> {
    let old_data = DATA.lock().await;
    if old_data.checksum.is_empty() {
        return Ok(true);
    }

    // check latest, if recents update in 10s
    let latest = land_dao::deployment::get_latest_one().await?;
    if latest.is_none() {
        return Ok(false);
    }
    let latest = latest.unwrap();
    let now = chrono::Utc::now().naive_utc();
    if now.signed_duration_since(latest.updated_at).num_seconds() < 10 {
        return Ok(true);
    }
    Ok(false)
}

#[instrument("[GW]")]
pub async fn generate() -> Result<()> {
    let start_time = tokio::time::Instant::now();
    // debug!("Generating");

    if !is_should_generate().await? {
        // debug!("Skip, cost:{:?}", start_time.elapsed().as_millis());
        return Ok(());
    }

    let dps = land_dao::deployment::list_active().await?;
    debug!("Active deployments: {:?}", dps.len());
    let (domain, _) = land_dao::settings::get_domain_settings().await?;
    let st = land_dao::storage::get_storage().await?;

    let mut items = vec![];
    let dp_len = dps.len();
    for dp in dps {
        let item = ConfItem {
            user_id: dp.user_id,
            project_id: dp.project_id,
            dl_url: st.build_url(&dp.storage_path)?,
            path: dp.storage_path,
            status: dp.deploy_status,
            md5: dp.storage_md5,
            domain: format!("{}.{}", dp.domain, domain),
            key: dp.task_id,
        };
        items.push(item);
    }
    let checksum = format!("{:x}", md5::compute(serde_json::to_string(&items)?));
    let mut old_data = DATA.lock().await;
    if old_data.checksum == checksum {
        /*debug!(
            "No change, cost:{:?}",
            start_time.elapsed().as_millis()
        );*/
        return Ok(());
    }
    let conf = ConfData {
        items,
        checksum: checksum.clone(),
    };
    *old_data = conf;

    info!(
        "Generated ok, len:{}, checksum:{:?}, cost:{:?}",
        dp_len,
        checksum,
        start_time.elapsed().as_millis(),
    );
    Ok(())
}
