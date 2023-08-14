use anyhow::{anyhow, Result};
use envconfig::Envconfig;
use once_cell::sync::{Lazy, OnceCell};
use opendal::services::Memory;
use opendal::Operator;
use tokio::sync::Mutex;
use tracing::debug;

pub mod local;
pub mod s3;

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "STORAGE_TYPE", default = "local")]
    pub type_name: String,
}

pub static STORAGE: OnceCell<Operator> = OnceCell::new();

pub static GLOBAL: Lazy<Mutex<Operator>> = Lazy::new(|| {
    let mut builder = Memory::default();
    builder.root("/tmp");
    let op = Operator::new(builder).unwrap().finish();
    Mutex::new(op)
});

#[tracing::instrument(name = "[STORAGE]")]
pub async fn init() -> Result<()> {
    let cfg = Config::init_from_env().unwrap();
    debug!("Init storage cfg: {:?}", cfg);
    let op = get_operator(cfg.type_name).await?;
    STORAGE.set(op).map_err(|_| anyhow!("set storage error"))?;
    Ok(())
}

/// get_operator returns the storage operator
pub async fn get_operator(type_name: String) -> Result<Operator> {
    match type_name.as_str() {
        "local" => {
            let op = local::init().await?;
            Ok(op)
        }
        "cloudflare-r2" => {
            let op = s3::init().await?;
            Ok(op)
        }
        _ => Err(anyhow!("unknown storage type: {}", type_name)),
    }
}

/// write writes the content to the storage
pub async fn write(name: &str, content: Vec<u8>) -> Result<()> {
    let op = STORAGE.get().unwrap();
    op.write(name, content).await?;
    Ok(())
}
