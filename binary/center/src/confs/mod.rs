use crate::settings::load_storage_settings;
use anyhow::Result;
use land_core::confdata::{EndpointConf, RouteConfItem};
use land_dao::project;
use lazy_static::lazy_static;
use md5::{Digest, Md5};
use tokio::sync::Mutex;
use tracing::{debug, error, info, Instrument};

pub mod runtime_node;

lazy_static! {
    pub static ref CONF_VALUES: Mutex<EndpointConf> = Mutex::new(EndpointConf {
        items: vec![],
        created_at: 0,
        md5: "".to_string(),
    });
}

async fn sync_conf_loop(interval: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval));
    loop {
        interval.tick().await;

        let mut conf_values = CONF_VALUES.lock().await;

        let should_generate = should_generate().await;
        if should_generate.is_err() {
            error!("should generate error: {:?}", should_generate.err());
            continue;
        }
        if !should_generate.unwrap() && conf_values.created_at > 0 {
            continue;
        }

        let new_conf_values = generate().await;
        match new_conf_values {
            Ok(new_conf_values) => {
                debug!("generate conf: {:?}", new_conf_values.md5);
                if conf_values.md5 != new_conf_values.md5 {
                    info!("update conf: {:?}", new_conf_values.md5);
                    *conf_values = new_conf_values;
                }
            }
            Err(e) => {
                error!("generate conf error: {:?}", e);
            }
        }
    }
}

pub fn run(interval: u64, runtime_node_interval: u64) {
    tokio::spawn(
        async move {
            sync_conf_loop(interval).await;
        }
        .instrument(tracing::info_span!("[CONFS]")),
    );
    tokio::spawn(
        async move {
            runtime_node::sync_runtime_node(runtime_node_interval).await;
        }
        .instrument(tracing::info_span!("[RTNODES]")),
    );
}

async fn should_generate() -> Result<bool> {
    let flag1 = land_dao::deployment::is_recent_updated().await?;
    let flag2 = land_dao::project::is_recent_updated().await?;
    Ok(flag1 || flag2)
}

async fn generate() -> Result<EndpointConf> {
    // get all available projects
    let projects = land_dao::project::list_all_available().await?;
    let projects_map = projects
        .into_iter()
        .map(|p| (p.id, p))
        .collect::<std::collections::HashMap<i32, land_dao::Project>>();

    // get all success deployments
    let deployments = land_dao::deployment::list_success().await?;
    let deployments_len = deployments.len();

    // get storage settings
    let (typename, _, s3_config) = load_storage_settings().await?;

    // provide build download url function
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

    // generate confs
    let d = crate::settings::DOMAIN.lock().await;
    let prod_domain = d.domain.clone();

    // generate route confs
    let mut conf_items = Vec::new();
    for deployment in deployments {
        let project = projects_map.get(&deployment.project_id);
        if project.is_none() {
            continue;
        }
        let project = project.unwrap();
        if project.status == project::Status::InActive.to_string() {
            continue;
        }

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
    debug!(
        "deployments: {}, items: {}",
        deployments_len,
        conf_items.len()
    );

    // use items's json value to generate md5 hash
    let json_bytes = serde_json::to_vec(&conf_items)?;
    let mut hasher = Md5::new();
    hasher.update(json_bytes);
    let result = hasher.finalize();
    let md5 = format!("{:x}", result);

    // confValues
    let conf = EndpointConf {
        items: conf_items,
        md5,
        created_at: chrono::Utc::now().timestamp() as u64,
    };

    Ok(conf)
}
