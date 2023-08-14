use anyhow::Result;
use envconfig::Envconfig;
use opendal::{services::S3, Operator};
use serde::{Deserialize, Serialize};

#[derive(Envconfig, Serialize, Deserialize, Debug)]
pub struct Config {
    #[envconfig(from = "S3_ENDPOINT")]
    endpoint: String,
    #[envconfig(from = "S3_BUCKET")]
    bucket: String,
    #[envconfig(from = "S3_REGION", default = "auto")]
    region: String,
    #[envconfig(from = "S3_ACCESS_KEY_ID")]
    access_key_id: String,
    #[envconfig(from = "S3_SECRET_ACCESS_KEY")]
    secret_access_key: String,
    #[envconfig(from = "S3_ROOT_PATH", default = "/wasm-bin")]
    root_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "".to_string(),
            bucket: "".to_string(),
            region: "auto".to_string(),
            access_key_id: "".to_string(),
            secret_access_key: "".to_string(),
            root_path: "/wasm-bin".to_string(),
        }
    }
}

pub async fn init() -> Result<Operator> {
    let cfg = Config::init_from_env()?;
    create(&cfg).await
}

pub async fn create(cfg: &Config) -> Result<Operator> {
    let mut builder = S3::default();
    builder.root(&cfg.root_path);
    builder.bucket(&cfg.bucket);
    builder.endpoint(&cfg.endpoint);
    builder.region(&cfg.region);
    builder.batch_max_operations(300); // cloudflare R2 need < 700
    builder.access_key_id(&cfg.access_key_id);
    builder.secret_access_key(&cfg.secret_access_key);

    let op: Operator = Operator::new(builder)?.finish();
    Ok(op)
}

/// reload_global reloads the global storage with the new config
pub async fn reload_global(cfg: &Config) -> Result<()> {
    let op = create(cfg).await?;
    let mut global = crate::GLOBAL.lock().await;
    *global = op;
    Ok(())
}
