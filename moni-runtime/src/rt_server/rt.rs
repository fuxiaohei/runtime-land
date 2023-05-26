use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use moka::sync::Cache;
use moni_lib::storage::STORAGE;
use moni_runtime::WorkerPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

lazy_static! {
    pub static ref WASM_POOL_INSTANCES: Cache<String,Arc<WorkerPool> > = Cache::builder()
    // Time to live (TTL): 24 hours
    .time_to_live(Duration::from_secs(24 * 60 * 60))
    // Time to idle (TTI):  1 hours
    .time_to_idle(Duration::from_secs(60 * 60))
    // Create the cache.
    .build();
}

pub async fn prepare_worker_pool(key: &str) -> Result<Arc<WorkerPool>> {
    let mut instances_pool = WASM_POOL_INSTANCES.get(key);

    // if pool is not exist, create it
    if instances_pool.is_none() {
        let storage = STORAGE.get().expect("storage not found");
        if !storage.is_exist(key).await? {
            return Err(anyhow!("key not found: {}", key));
        }
        let binary = storage.read(key).await?;

        // write binary to local file
        let mut path = std::env::temp_dir();
        path.push(key);
        // create parent dir
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        std::fs::write(&path, binary)?;
        debug!("wasm temp binary write to {}", path.display());

        // create wasm worker pool
        let pool = moni_runtime::create_pool(path.to_str().unwrap())?;
        WASM_POOL_INSTANCES.insert(key.to_string(), Arc::new(pool));

        instances_pool = WASM_POOL_INSTANCES.get(key);
    }

    Ok(instances_pool.unwrap())
}
