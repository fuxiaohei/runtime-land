use anyhow::anyhow;
use anyhow::Result;
use async_zip::tokio::read::seek::ZipFileReader;
use futures_util::AsyncReadExt;
use land_core::storage;
use land_worker::Worker;
use lazy_static::lazy_static;
use moka::sync::Cache;
use std::io::Cursor;
use std::time::Duration;
use tokio::time::Instant;
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
    let suffix = key.split('.').last().unwrap();
    if suffix == "zip" {
        return prepare_zip_worker(key).await;
    }
    prepare_wasm_worker(key).await
}

async fn prepare_wasm_worker(key: &str) -> Result<Worker> {
    let st = Instant::now();
    let mut instances_pool = WASM_INSTANCES.get(key);

    if let Some(instance) = instances_pool {
        return Ok(instance);
    }

    if !storage::is_exist(key).await? {
        return Err(anyhow!("key not found: {}", key));
    }
    let binary = storage::read(key).await?;

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

async fn prepare_zip_worker(key: &str) -> Result<Worker> {
    let st = Instant::now();
    let wasm_key = key.replace(".zip", ".wasm");

    // check if wasm file exists
    let mut instances_pool = WASM_INSTANCES.get(&wasm_key);

    if let Some(instance) = instances_pool {
        return Ok(instance);
    }

    // read wasm from zip file
    let binary = storage::read(key).await?;
    let mut file = Cursor::new(binary);
    let mut reader = ZipFileReader::with_tokio(&mut file).await?;
    let mut buf = Vec::new();
    // let mut bundle_wasm_file = archive.by_name("bundle.wasm")?;
    for index in 0..reader.file().entries().len() {
        let entry = reader.file().entries().get(index).unwrap();
        let entry_name = entry.entry().filename().as_str().unwrap();
        if entry_name == "bundle.wasm" {
            let mut entry_reader = reader
                .reader_without_entry(index)
                .await
                .expect("Failed to read ZipEntry");

            entry_reader.read_to_end(&mut buf).await?;
            break;
        }
    }

    // create wasm worker pool
    let worker = Worker::from_binary(&buf).await?;
    WASM_INSTANCES.insert(wasm_key.clone(), worker);

    instances_pool = WASM_INSTANCES.get(&wasm_key);
    info!(elapsed = %st.elapsed().as_micros(),"Worker created");

    Ok(instances_pool.unwrap())
}
