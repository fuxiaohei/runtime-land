use anyhow::Result;
use envconfig::Envconfig;
use opendal::services::Fs;
use opendal::Operator;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Envconfig, Serialize, Deserialize, Debug)]
pub struct Config {
    #[envconfig(from = "FS_PATH", default = "/tmp/runtime-land-data")]
    pub path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: "/tmp/runtime-land-data".to_string(),
        }
    }
}

/// init_from_env init local storage from env
pub async fn init_from_env() -> Result<Operator> {
    let cfg = Config::init_from_env()?;
    build(&cfg).await
}

/// create creates the local storage
pub async fn build(cfg: &Config) -> Result<Operator> {
    let mut builder = Fs::default();
    builder.root(&cfg.path);
    let op: Operator = Operator::new(builder)?.finish();
    info!("Init local storage success, path: {}", cfg.path);
    Ok(op)
}
