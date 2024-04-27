use super::Worker;
use crate::engine::MODULE_VERSION;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use moka::sync::Cache;
use once_cell::sync::OnceCell;
use std::time::Duration;
use tokio::time::Instant;
use tracing::{debug, info, warn};

/// FILE_DIR is the directory of wasm files
pub static FILE_DIR: OnceCell<String> = OnceCell::new();

lazy_static! {
    pub static ref WASM_INSTANCES: Cache<String,Worker > = Cache::builder()
    // Time to live (TTL): 3 hours
    .time_to_live(Duration::from_secs(3 * 60 * 60))
    // Time to idle (TTI):  60 minutes
    .time_to_idle(Duration::from_secs(60 * 60))
    // Create the cache.
    .build();
}

async fn prepare_wasm_worker(key: &str, is_aot: bool) -> Result<Worker> {
    let st = Instant::now();
    let dir = FILE_DIR.get().unwrap();
    let real_file = format!("{}/{}", dir, key);

    if !std::path::Path::new(&real_file).exists() {
        warn!("Wasm file not found: {}", real_file);
        return Err(anyhow!("Function is not found"));
    }

    // create wasm worker pool
    let worker = Worker::new(&real_file, is_aot).await?;
    WASM_INSTANCES.insert(key.to_string(), worker);

    let instances_pool = WASM_INSTANCES.get(key);
    info!(elapsed = %st.elapsed().as_millis(),"Worker created");

    Ok(instances_pool.unwrap())
}

pub async fn prepare_worker(key: &str, is_aot: bool) -> Result<Worker> {
    let instances_pool = WASM_INSTANCES.get(key);
    if let Some(instance) = instances_pool {
        return Ok(instance);
    }
    if key.ends_with(".wasm") {
        return prepare_wasm_worker(key, is_aot).await;
    }
    Err(anyhow!("Invalid key"))
}

/// compile_aot compile aot wasm
pub async fn compile_aot(path: &str) -> Result<()> {
    let suffix = format!(".wasm.{}.aot", MODULE_VERSION);
    let aot_path = path.replace(".wasm", &suffix);
    if std::path::Path::new(&aot_path).exists() {
        debug!("AOT file already exists: {}", &aot_path);
        return Ok(());
    }
    Worker::compile_aot(path, &aot_path)?;
    debug!("Compile AOT success: {}", &aot_path);
    Ok(())
}
