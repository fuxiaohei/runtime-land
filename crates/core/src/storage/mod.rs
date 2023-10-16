use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use opendal::services::Memory;
use opendal::{Metadata, Operator};
use tokio::sync::Mutex;

mod fs;
mod s3;

/// build returns the storage operator
pub async fn build(type_name: &str) -> Result<Operator> {
    match type_name {
        "fs" => {
            let op = fs::init_from_env().await?;
            Ok(op)
        }
        "s3" => {
            let op = s3::init_from_env().await?;
            Ok(op)
        }
        _ => Err(anyhow!("unknown storage type: {}", type_name)),
    }
}

/// GLOBAL is the global storage operator
pub static GLOBAL: Lazy<Mutex<Operator>> = Lazy::new(|| {
    let mut builder = Memory::default();
    builder.root("/tmp");
    let op = Operator::new(builder).unwrap().finish();
    Mutex::new(op)
});

/// init_from_type init storage from type
#[tracing::instrument(name = "[STORAGE]")]
pub async fn build_global(typename: &str) -> Result<()> {
    let op = build(typename).await?;
    let mut global = GLOBAL.lock().await;
    *global = op;
    Ok(())
}

/// write_global writes the content to the global storage
pub async fn write(name: &str, content: Vec<u8>) -> Result<()> {
    let op = GLOBAL.lock().await;
    op.write(name, content).await?;
    Ok(())
}

/// is_exist checks if the file exists
pub async fn is_exist(name: &str) -> Result<bool> {
    let op = GLOBAL.lock().await;
    Ok(op.is_exist(name).await?)
}

/// read reads the content from the storage
pub async fn read(name: &str) -> Result<Vec<u8>> {
    let op = GLOBAL.lock().await;
    Ok(op.read(name).await?)
}

/// delete deletes the file from the storage
pub async fn delete(name: &str) -> Result<()> {
    let op = GLOBAL.lock().await;
    op.delete(name).await?;
    Ok(())
}

/// stat returns the storage stats
pub async fn stat(name: &str) -> Result<Metadata> {
    let op = GLOBAL.lock().await;
    let metadata = op.stat(name).await?;
    Ok(metadata)
}
