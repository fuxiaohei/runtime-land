use anyhow::Result;
use land_common::obj_hash;
use land_dao::{deploys, settings, store};
use lazy_static::lazy_static;
use tokio::{sync::Mutex, time::Instant};
use tracing::{debug, instrument, warn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub user_id: i32,
    pub project_id: i32,
    pub deploy_id: i32,
    pub task_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_hash: String,
    pub domain: String,
}

/// init_confs is used to generate confs in background
pub async fn init_confs() {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            match gen().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("confs::gen error: {}", e);
                }
            }
        }
    });
}

lazy_static! {
    static ref CONFS: Mutex<(String, Vec<Item>)> = Mutex::new(("".to_string(), vec![]));
}

/// gen generate config
#[instrument("[AGENT-CONFS]")]
pub async fn gen() -> anyhow::Result<()> {
    let st = Instant::now();
    let ids = deploys::success_ids().await?;
    if ids.is_empty() {
        return Ok(());
    }
    let ids_hash = obj_hash(ids.clone())?;
    let mut confs = CONFS.lock().await;
    if confs.0 == ids_hash {
        // debug!("No changed");
        return Ok(());
    }
    confs.0.clone_from(&ids_hash);
    confs.1 = gen_confs(ids).await?;
    let elasped = st.elapsed().as_millis();
    debug!("Generated in {}ms, hash: {}", elasped, ids_hash);
    Ok(())
}

async fn gen_confs(ids: Vec<i32>) -> Result<Vec<Item>> {
    let domain_settings = settings::get_domain_settings().await?;

    // get deploys data
    let deploy_data = deploys::list_by_ids(ids.clone()).await?;
    let storage_data = store::list_success_by_deploys(ids).await?;

    // build confs
    let mut items = Vec::new();
    for deploy in deploy_data {
        let storage_item = storage_data.get(&deploy.id);
        if storage_item.is_none() {
            warn!("Storage not found for deploy {}", deploy.id);
            continue;
        }
        let storage_item = storage_item.unwrap();
        let domain = format!("{}.{}", deploy.domain, domain_settings.domain_suffix);
        let item = Item {
            user_id: deploy.owner_id,
            project_id: deploy.project_id,
            deploy_id: deploy.id,
            task_id: deploy.task_id.clone(),
            file_name: storage_item.path.clone(),
            download_url: storage_item.file_target.clone(),
            file_hash: storage_item.file_hash.clone(),
            domain,
        };
        items.push(item);
    }
    Ok(items)
}

/// get_confs get config
pub async fn get_confs() -> (String, Vec<Item>) {
    CONFS.lock().await.clone()
}
