use anyhow::Result;
use envconfig::Envconfig;
use futures_util::StreamExt;
use once_cell::sync::OnceCell;
use opendal::Operator;
use tracing::debug;

static LOCAL_STORE: OnceCell<Operator> = OnceCell::new();
static REMOTE_STORE: OnceCell<Operator> = OnceCell::new();
pub static REMOTE_STORE_ENABLED: OnceCell<bool> = OnceCell::new();

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "LOCAL_STORE_TYPE", default = "local")]
    pub local_store_type: String,
    #[envconfig(from = "REMOTE_STORE_TYPE", default = "cloudflare-r2")]
    pub remote_store_type: String,
    #[envconfig(from = "REMOTE_STORE_ENABLED", default = "true")]
    pub remote_store_enabled: bool,
}

pub async fn init() -> Result<()> {
    let cfg = Config::init_from_env().unwrap();
    debug!("Init storage cfg: {:?}", cfg);

    let local_op = land_storage::get_operator(cfg.local_store_type).await?;
    LOCAL_STORE
        .set(local_op)
        .map_err(|_| anyhow::anyhow!("set local store error"))?;

    if cfg.remote_store_enabled {
        let remote_op = land_storage::get_operator(cfg.remote_store_type).await?;
        REMOTE_STORE
            .set(remote_op)
            .map_err(|_| anyhow::anyhow!("set remote store error"))?;
        REMOTE_STORE_ENABLED.set(true).unwrap();
    } else {
        REMOTE_STORE_ENABLED.set(false).unwrap();
    }

    Ok(())
}

/// save_remote_to_local saves the remote file to local
pub async fn save_remote_to_local(path: &str) -> Result<()> {
    if REMOTE_STORE_ENABLED.get().unwrap() == &false {
        return Ok(());
    }
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
