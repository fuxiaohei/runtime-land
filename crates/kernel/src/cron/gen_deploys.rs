use anyhow::Result;
use land_dao::{deployment, settings, storage};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, info, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfItem {
    pub user_id: i32,
    pub project_id: i32,
    pub path: String,
    pub dl_url: String,
    pub status: String,
    pub md5: String,
    pub domain: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfData {
    pub items: Vec<ConfItem>,
    pub checksum: String,
}

/// DATA is a global variable to store deployment data
static DATA: Lazy<Mutex<ConfData>> = Lazy::new(|| {
    Mutex::new(ConfData {
        items: vec![],
        checksum: "".to_string(),
    })
});

pub async fn get() -> ConfData {
    let data = DATA.lock().await;
    data.clone()
}

#[instrument("[CRON-gen-deploys]")]
pub async fn cron() {
    let start_time = tokio::time::Instant::now();

    // check if need to generate
    let ok = match is_need_gen().await {
        Ok(ok) => ok,
        Err(e) => {
            info!(
                "Check need gen error: {:?}, cost:{:?}ms",
                e,
                start_time.elapsed().as_millis()
            );
            return;
        }
    };
    if !ok {
        //debug!("Skip, cost:{:?}ms", start_time.elapsed().as_millis());
        return;
    }

    let data = match gen().await {
        Ok(data) => data,
        Err(e) => {
            info!(
                "Generate error: {:?}, cost:{:?}ms",
                e,
                start_time.elapsed().as_millis()
            );
            return;
        }
    };

    let mut old = DATA.lock().await;
    if old.checksum == data.checksum {
        debug!("No change, cost:{:?}ms", start_time.elapsed().as_millis());
        return;
    }
    info!(
        "Done, cost:{:?}ms, checksum:{}",
        start_time.elapsed().as_millis(),
        data.checksum
    );

    *old = data;
}

async fn is_need_gen() -> Result<bool> {
    // if data is empty, need to generate
    let old = DATA.lock().await;
    if old.checksum.is_empty() {
        return Ok(true);
    }

    // get lastest deployment, if recents update in 10s
    let latest = deployment::get_latest_one().await?;
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

pub async fn gen() -> Result<ConfData> {
    let dps = deployment::list_active().await?;
    let (domain, _) = settings::get_domain_settings().await?;
    let st = storage::get_storage().await?;

    let mut items = vec![];
    for dp in dps {
        if dp.storage_path.is_empty() {
            continue;
        }
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
    Ok(ConfData { items, checksum })
}
