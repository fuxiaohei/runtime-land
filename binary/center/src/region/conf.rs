use crate::settings::{self, load_storage_settings};
use anyhow::Result;
use land_core::confdata::{RouteConfItem, RoutesConf};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, Instrument};

#[derive(Debug)]
struct OperationFlag {
    flag: bool,
    trigger_at: u64,
    last_trigger_at: u64,
}

lazy_static! {
    static ref OPERATION_FLAG: Mutex<OperationFlag> = Mutex::new(OperationFlag {
        flag: true,
        trigger_at: 0,
        last_trigger_at: 0,
    });
}

lazy_static! {
    pub static ref CONF_VALUES: Mutex<RoutesConf> = Mutex::new(RoutesConf {
        items: vec![],
        created_at: 0,
    });
}

/// trigger conf build
pub async fn trigger() {
    let mut flag = OPERATION_FLAG.lock().await;
    flag.flag = true;
    flag.trigger_at = chrono::Utc::now().timestamp() as u64;
    info!("trigger conf, flag: {:?}", flag);
}

/// init conf building loop
pub async fn init() {
    // use 1s seconds to refresh conf values
    tokio::spawn(
        async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let mut flag = OPERATION_FLAG.lock().await;
                if flag.flag {
                    flag.flag = false;
                    flag.trigger_at = 0;
                    flag.last_trigger_at = chrono::Utc::now().timestamp() as u64;
                    info!("trigger build conf, flag: {:?}", flag);
                    match build_conf().await {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("build conf error: {:?}", e);
                        }
                    }
                }
            }
        }
        .instrument(tracing::info_span!("[CONF]")),
    );
}

async fn build_conf() -> Result<()> {
    let deployments = land_dao::deployment::list_success().await.unwrap();

    debug!("deployments: {:?}", deployments.len());

    let (typename, _, s3_config) = load_storage_settings().await?;

    let build_download_url = |path: &str| -> String {
        match typename.as_str() {
            "s3" => {
                format!(
                    "{}/{}/{}",
                    s3_config.bucket_basepath.trim_end_matches('/'),
                    s3_config.root_path.trim_start_matches('/'),
                    path
                )
            }
            _ => path.to_string(),
        }
    };

    let d = settings::DOMAIN.lock().await;
    let prod_domain = d.domain.clone();

    let mut conf_items = Vec::new();
    for deployment in deployments {
        let conf_item = RouteConfItem::new(
            format!("{}.{}", deployment.domain, prod_domain),
            deployment.storage_path.clone(),
            deployment.uuid,
            build_download_url(&deployment.storage_path),
            deployment.updated_at.timestamp() as u64,
        );
        conf_items.push(conf_item);
        if !deployment.prod_domain.is_empty() {
            let conf_item = RouteConfItem::new(
                format!("{}.{}", deployment.prod_domain, prod_domain),
                deployment.storage_path.clone(),
                format!("{}-prod", deployment.project_uuid),
                build_download_url(&deployment.storage_path),
                deployment.updated_at.timestamp() as u64,
            );
            conf_items.push(conf_item);
        }
    }

    debug!("conf_items: {:?}", conf_items.len());

    let mut conf_values = CONF_VALUES.lock().await;
    conf_values.items = conf_items;
    conf_values.created_at = chrono::Utc::now().timestamp_nanos() as u64;

    Ok(())
}
