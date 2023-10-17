use land_core::confdata::RuntimeNodeInfo;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{debug, info};

#[derive(Debug)]
pub struct RuntimeNodeData {
    info: RuntimeNodeInfo,
    conf_md5: String,
}

/// global runtime node data map
pub static RUNTIME_NODES_MAP: Lazy<Mutex<HashMap<String, RuntimeNodeData>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// update_runtime_node_data update runtime node data
pub async fn update_data(info: RuntimeNodeInfo, conf_md5: String) {
    let data = RuntimeNodeData { info, conf_md5 };
    let mut map = RUNTIME_NODES_MAP.lock().await;
    map.insert(data.info.region_ip(), data);
}

/// sync_runtime_node sync runtime node data with db
pub async fn sync_runtime_node(interval: u64) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
    loop {
        compare_and_update().await;
        interval.tick().await;
    }
}

async fn compare_and_update() {
    let db_nodes = land_dao::runtime_node::get_all_map().await.unwrap();
    let mut current_nodes = RUNTIME_NODES_MAP.lock().await;
    debug!(
        "db nodes: {}, current nodes: {}",
        db_nodes.len(),
        current_nodes.len()
    );

    // iterator current nodes map
    for (key, data) in current_nodes.iter_mut() {
        // if key not in db nodes, create new one
        if !db_nodes.contains_key(key) {
            info!("create new runtime node: {}", data.info.region_ip());
            land_dao::runtime_node::create(data.info.clone(), data.conf_md5.clone())
                .await
                .unwrap();
            continue;
        }

        info!("update runtime node: {}", data.info.region_ip());
        land_dao::runtime_node::update_online(key.clone(), data.conf_md5.clone())
            .await
            .unwrap();
    }

    // update offline nodes
    land_dao::runtime_node::update_offline(180).await.unwrap(); // 3min

    // after iterator, current nodes can clear
    current_nodes.clear();
}
