use anyhow::Result;
use once_cell::sync::OnceCell;
use opendal::Operator;

pub static LOCAL_STORE: OnceCell<Operator> = OnceCell::new();

pub async fn init() -> Result<()> {
    let local_op = land_storage::build_operator("fs").await?;
    LOCAL_STORE
        .set(local_op)
        .map_err(|_| anyhow::anyhow!("set local store error"))?;

    Ok(())
}

/// is_exist checks if the file exists
pub async fn is_exist(name: &str) -> Result<bool> {
    let op = LOCAL_STORE.get().unwrap();
    Ok(op.is_exist(name).await?)
}

/// read reads the content from the storage
pub async fn read(name: &str) -> Result<Vec<u8>> {
    let op = LOCAL_STORE.get().unwrap();
    Ok(op.read(name).await?)
}
