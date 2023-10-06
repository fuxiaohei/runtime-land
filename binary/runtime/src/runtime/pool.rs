use anyhow::anyhow;
use anyhow::Result;
use land_core::storage;
use land_worker::Worker;
use lazy_static::lazy_static;
use moka::sync::Cache;
use std::time::Duration;
use tracing::{debug, info};

lazy_static! {
    pub static ref WASM_INSTANCES: Cache<String,Worker > = Cache::builder()
    // Time to live (TTL): 3 hours
    .time_to_live(Duration::from_secs(3* 60 * 60))
    // Time to idle (TTI):  60 minutes
    .time_to_idle(Duration::from_secs(60 * 60))
    // Create the cache.
    .build();
}

pub async fn prepare_worker(key: &str) -> Result<Worker> {
    let mut instances_pool = WASM_INSTANCES.get(key);

    if let Some(instance) = instances_pool {
        return Ok(instance);
    }

    if !storage::is_exist(key).await? {
        return Err(anyhow!("pool key not found: {}", key));
    }
    let binary = storage::read(key).await?;

    // write binary to local file
    let mut path = std::env::temp_dir();
    path.push(key);
    // create parent dir
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    std::fs::write(&path, binary)?;
    debug!("wasm temp binary write to {}", path.display());

    // create wasm worker pool
    let worker = Worker::new(path.to_str().unwrap()).await?;
    WASM_INSTANCES.insert(key.to_string(), worker);

    instances_pool = WASM_INSTANCES.get(key);
    info!("worker pool created");

    Ok(instances_pool.unwrap())
}
