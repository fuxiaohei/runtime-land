use anyhow::{Ok, Result};
use once_cell::sync::OnceCell;
use opendal::services::Fs;
use opendal::Operator;

static FS_DIR: OnceCell<Operator> = OnceCell::new();

/// init_fs initializes the storage
pub fn init_fs(dir: &str) -> Result<()> {
    let mut builder = Fs::default();
    builder.root(dir);
    let op: Operator = Operator::new(builder)?.finish();
    FS_DIR.set(op).unwrap();
    Ok(())
}

/// is_exist checks if the file exists in the storage
pub async fn is_exist(name: &str) -> Result<bool> {
    let fs = FS_DIR.get().unwrap();
    Ok(fs.is_exist(name).await?)
}

/// read reads the content from the storage
pub async fn read(name: &str) -> Result<Vec<u8>> {
    let op = FS_DIR.get().unwrap();
    Ok(op.read(name).await?)
}