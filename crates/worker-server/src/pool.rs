use super::fs;
use anyhow::{anyhow, Result};
use land_worker::Worker;
use lazy_static::lazy_static;
use moka::sync::Cache;
use tokio::time::{Duration, Instant};
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

/// prepare_worker prepares the worker
pub async fn prepare_worker(key: &str) -> Result<Worker> {
    if !key.ends_with(".wasm") {
        return Err(anyhow!("key must end with .wasm"));
    }
    prepare_wasm_worker(key).await
}

async fn prepare_wasm_worker(key: &str) -> Result<Worker> {
    let st = Instant::now();
    let mut instances_pool = WASM_INSTANCES.get(key);

    if let Some(instance) = instances_pool {
        return Ok(instance);
    }

    if !fs::is_exist(key).await? {
        return Err(anyhow!("key not found: {}", key));
    }
    let binary = fs::read(key).await?;

    // write binary to local file
    let mut path = std::env::temp_dir();
    path.push(key);
    // create parent dir
    let parent = path.parent().unwrap();
    tokio::fs::create_dir_all(parent).await?;
    tokio::fs::write(&path, binary).await?;
    debug!("Temp binary write to {}", path.display());

    // create wasm worker pool
    let worker = Worker::new(path.to_str().unwrap()).await?;
    WASM_INSTANCES.insert(key.to_string(), worker);

    instances_pool = WASM_INSTANCES.get(key);
    info!(elapsed = %st.elapsed().as_micros(),"Worker created");

    Ok(instances_pool.unwrap())
}
