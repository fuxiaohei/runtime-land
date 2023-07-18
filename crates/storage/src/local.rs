use anyhow::Result;
use envconfig::Envconfig;
use opendal::services::Fs;
use opendal::Operator;
use tracing::info;

#[derive(Envconfig, Debug)]
pub struct LocalConfig {
    #[envconfig(from = "STORAGE_LOCAL_PATH", default = "/tmp/runtime-land-data")]
    pub path: String,
}

pub async fn init() -> Result<Operator> {
    let cfg = LocalConfig::init_from_env()?;
    let mut builder = Fs::default();
    builder.root(&cfg.path);
    let op: Operator = Operator::new(builder)?.finish();
    info!("Init local storage success, path: {}", cfg.path);
    Ok(op)
}
