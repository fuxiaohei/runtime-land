use anyhow::Result;
use futures_util::StreamExt;
use once_cell::sync::OnceCell;
use opendal::Operator;
use tracing::debug;

use crate::center;

pub static LOCAL_STORE: OnceCell<Operator> = OnceCell::new();

pub async fn init() -> Result<()> {
    let local_op = land_storage::build_operator("fs").await?;
    LOCAL_STORE
        .set(local_op)
        .map_err(|_| anyhow::anyhow!("set local store error"))?;

    Ok(())
}

/// download_file saves the remote file to local
pub async fn download_file(download_url: &str, path: &str) -> Result<()> {
    // if local file exist, skip download
    let local_op = LOCAL_STORE.get().unwrap();
    if local_op.is_exist(path).await? {
        debug!("local file exist, path: {}", path);
        return Ok(());
    }
    let resp = center::request_file(download_url).await?;
    let mut reader = resp.bytes_stream();
    let mut writer = local_op.writer(path).await?;
    while let Some(bytes) = reader.next().await {
        let bytes = bytes?;
        writer.write(bytes).await?;
    }
    writer.close().await?;
    debug!("save remote to local success, path: {}", path);
    Ok(())
}

/// remove_local removes the local file
pub async fn remove_local(path: &str) -> Result<()> {
    let local_op = LOCAL_STORE.get().unwrap();
    local_op.delete(path).await?;
    Ok(())
}
