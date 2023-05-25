use anyhow::Result;
use envconfig::Envconfig;
use opendal::services::Fs;
use opendal::Operator;

#[derive(Envconfig, Debug)]
pub struct LocalConfig {
    #[envconfig(from = "MONI_STORAGE_LOCAL_PATH", default = "/tmp/moni")]
    pub path: String,
}

pub async fn init_local() -> Result<Operator> {
    let cfg = LocalConfig::init_from_env()?;

    let mut builder = Fs::default();
    builder.root(&cfg.path);
    let op: Operator = Operator::new(builder)?.finish();

    Ok(op)
}
