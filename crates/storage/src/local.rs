use anyhow::Result;
use envconfig::Envconfig;
use opendal::services::Fs;
use opendal::Operator;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Envconfig, Serialize, Deserialize, Debug)]
pub struct Config {
    #[envconfig(from = "STORAGE_LOCAL_PATH", default = "/tmp/runtime-land-data")]
    pub path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: "/tmp/runtime-land-data".to_string(),
        }
    }
}

pub async fn init() -> Result<Operator> {
    let cfg = Config::init_from_env()?;
    create(&cfg).await
}

/// create creates the local storage
pub async fn create(cfg: &Config) -> Result<Operator> {
    let mut builder = Fs::default();
    builder.root(&cfg.path);
    let op: Operator = Operator::new(builder)?.finish();
    info!("Init local storage success, path: {}", cfg.path);
    Ok(op)
}

/// reload_global reloads the global storage with the new config
pub async fn reload_global(cfg: &Config) -> Result<()> {
    let op = create(cfg).await?;
    let mut global = crate::GLOBAL.lock().await;
    *global = op;
    Ok(())
}
