use anyhow::Result;
use futures_util::StreamExt;
use land_storage::local;
use land_storage::s3;
use once_cell::sync::OnceCell;
use opendal::Operator;
use tracing::debug;

static LOCAL_STORE: OnceCell<Operator> = OnceCell::new();
static REMOTE_STORE: OnceCell<Operator> = OnceCell::new();

pub async fn init() -> Result<()> {
    let local_op = local::init().await?;
    LOCAL_STORE
        .set(local_op)
        .map_err(|_| anyhow::anyhow!("set local store error"))?;

    let remote_op = s3::init().await?;
    REMOTE_STORE
        .set(remote_op)
        .map_err(|_| anyhow::anyhow!("set remote store error"))?;

    Ok(())
}

/// save_remote_to_local saves the remote file to local
pub async fn save_remote_to_local(path: &str) -> Result<()> {
    let remote_op = REMOTE_STORE.get().unwrap();
    let local_op = LOCAL_STORE.get().unwrap();

    let mut reader = remote_op.reader(path).await?;
    let mut writer = local_op.writer(path).await?;
    // reader is Stream<Item = <io::Result<Bytes>>>, so write bytes to writer via pipeline
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
