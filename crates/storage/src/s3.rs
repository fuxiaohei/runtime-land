use anyhow::Result;
use envconfig::Envconfig;
use opendal::{services::S3, Operator};
use serde::{Deserialize, Serialize};

#[derive(Envconfig, Serialize, Deserialize, Debug)]
pub struct Config {
    #[envconfig(from = "S3_ENDPOINT")]
    pub endpoint: String,
    #[envconfig(from = "S3_BUCKET")]
    pub bucket: String,
    #[envconfig(from = "S3_REGION", default = "auto")]
    pub region: String,
    #[envconfig(from = "S3_ACCESS_KEY_ID")]
    pub access_key_id: String,
    #[envconfig(from = "S3_SECRET_ACCESS_KEY")]
    pub secret_access_key: String,
    #[envconfig(from = "S3_ROOT_PATH", default = "/wasm-bin")]
    pub root_path: String,
    #[envconfig(from = "S3_BUCKET_BASEPATH")]
    pub bucket_basepath: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // this is the default value, not available in reality
            endpoint: "https://s3.us-east-2.amazonaws.com".to_string(),
            bucket: "demo-bucket".to_string(),
            region: "auto".to_string(),
            access_key_id: "access_key_id".to_string(),
            secret_access_key: "secret_access_key".to_string(),
            root_path: "/wasm-bin".to_string(),
            bucket_basepath: "https://s3.us-east-2.amazonaws.com".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let cfg = Config::init_from_env()?;
        Ok(cfg)
    }
    pub fn validate(&self) -> Result<()> {
        if self.endpoint.is_empty() {
            return Err(anyhow::anyhow!("S3_ENDPOINT is empty"));
        }
        if self.bucket.is_empty() {
            return Err(anyhow::anyhow!("S3_BUCKET is empty"));
        }
        if self.region.is_empty() {
            return Err(anyhow::anyhow!("S3_REGION is empty"));
        }
        if self.access_key_id.is_empty() {
            return Err(anyhow::anyhow!("S3_ACCESS_KEY_ID is empty"));
        }
        if self.secret_access_key.is_empty() {
            return Err(anyhow::anyhow!("S3_SECRET_ACCESS_KEY is empty"));
        }
        Ok(())
    }
}

pub async fn init() -> Result<Operator> {
    let cfg = Config::new()?;
    create(&cfg).await
}

pub async fn create(cfg: &Config) -> Result<Operator> {
    cfg.validate()?;
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
    cfg.validate()?;
    let op = create(cfg).await?;
    let mut global = crate::GLOBAL.lock().await;
    *global = op;
    Ok(())
}
