use anyhow::Result;
use once_cell::sync::Lazy;
use opendal::services::Memory;
use opendal::Operator;
use tokio::sync::Mutex;

mod fs;
pub use fs::reload_global as reload_fs_global;
pub use fs::Config as FsConfig;

mod s3;
pub use s3::reload_global as reload_s3_global;
pub use s3::Config as S3Config;

pub mod dao;

/// GLOBAL is the global storage operator
pub static GLOBAL: Lazy<Mutex<Operator>> = Lazy::new(|| {
    let mut builder = Memory::default();
    builder.root("/tmp");
    let op = Operator::new(builder).unwrap().finish();
    Mutex::new(op)
});

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
