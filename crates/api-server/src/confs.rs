use anyhow::Result;
use land_dblayer::{deployment, settings, storage};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, Instrument};

pub fn init_loop() -> Result<()> {
    tokio::spawn(async move {
        // run loop_once in background and every 1 second
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if let Err(err) = loop_once().await {
                error!("confs loop_once error: {:?}", err);
            }
        }
    });
    Ok(())
}

async fn loop_once() -> Result<()> {
    let span = tracing::info_span!("confs_loop");

    let mut confs = CONFS.lock().await;
    // no confs, build it
    if confs.routes_md5.is_empty() {
        debug!("loop-by-empty");
        let new_confs = build_confs().instrument(span.clone()).await?;
        *confs = new_confs;
        return Ok(());
    }

    // check flag
    let now_ts = chrono::Utc::now().timestamp();
    let trigger_ts = settings::get_confs_refresh_flag().await?;
    if now_ts - trigger_ts <= 10 {
        debug!("loop-by-trigger");
        // if trigger_ts is less than 10 seconds, means need refresh
        let new_confs = build_confs().instrument(span.clone()).await?;
        *confs = new_confs;
        return Ok(());
    }

    // check outdate
    // if no changes in 10 minutes, refresh it
    if now_ts - confs.created_at > 600 {
        debug!("loop-by-outdate");
        let new_confs = build_confs().instrument(span.clone()).await?;
        *confs = new_confs;
        return Ok(());
    }

    // check latest updated
    // if latest updates are in recent 10 seconds, refresh it
    let deploys = deployment::get_latest_updated(10).await?;
    // debug!("get_latest_updated: {:?}", deploys.len());
    if deploys.is_empty() {
        return Ok(());
    }
    debug!("loop-by-deploys");
    let new_confs = build_confs().instrument(span.clone()).await?;
    *confs = new_confs;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteItem {
    pub project_id: i32,
    pub project_name: String,
    pub owner_id: i32,
    pub uuid: String,
    pub route: String,
    pub module_path: String,
    pub resource_path: String,
    pub resource_md5: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfData {
    pub routes_md5: String,
    pub routes: Vec<RouteItem>,
    pub created_at: i64,
}

/// CONFS is the global confs data
pub static CONFS: Lazy<Mutex<ConfData>> = Lazy::new(|| {
    let op = ConfData {
        routes_md5: "".to_string(),
        routes: vec![],
        created_at: 0,
    };
    Mutex::new(op)
});

async fn build_confs() -> Result<ConfData> {
    let (domain_suffx, protocol) = settings::get_domain_settings().await?;
    let storage = storage::get_storage().await?;

    let deploys = deployment::list_actives().await?;
    info!("list_actives: {:?}", deploys.len());

    let mut routes = vec![];
    for deploy in deploys {
        let route = format!("{}://{}.{}/", protocol, deploy.name, domain_suffx);
        let route_item = RouteItem {
            project_id: deploy.project_id,
            project_name: deploy.project_name,
            owner_id: deploy.owner_id,
            uuid: deploy.trace_uuid,
            route,
            resource_path: storage.build_url(&deploy.storage_path)?,
            resource_md5: deploy.storage_md5,
            module_path: deploy.storage_path,
        };
        debug!("route_item: {:?}", route_item);
        routes.push(route_item);
    }

    let routes_json = serde_json::to_string(&routes)?;
    let routes_md5 = format!("{:x}", md5::compute(routes_json));
    let created_at = chrono::Utc::now().timestamp();
    let current_confs = ConfData {
        routes_md5,
        routes,
        created_at,
    };
    info!("confs md5: {:?}", current_confs.routes_md5);
    Ok(current_confs)
}
