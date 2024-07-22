use super::UrlBuilder;
use anyhow::Result;
use land_common::obj_hash;
use land_dao::settings;
use opendal::{services::Fs, Operator};
use serde::{Deserialize, Serialize};
use tracing::debug;

static FS_SETTINGS: &str = "storage-fs";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub local_path: String,
    pub local_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            local_path: "./data".to_string(),
            local_url: "/download/{name}".to_string(),
        }
    }
}

impl UrlBuilder for Settings {
    fn build_url(&self, name: &str) -> String {
        self.local_url.replace("{name}", name)
    }
}

/// init_defaults init default values for fs storage settings
pub async fn init_defaults() -> Result<()> {
    let settings: Option<Settings> = settings::get(FS_SETTINGS).await?;
    if settings.is_none() {
        let setting = Settings::default();
        settings::set(FS_SETTINGS, &setting).await?;
        debug!("init fs storage settings: {:?}", setting);
    }
    Ok(())
}

/// get get fs storage settings
pub async fn get() -> Result<Settings> {
    let settings: Option<Settings> = settings::get(FS_SETTINGS).await?;
    if settings.is_none() {
        return Err(anyhow::anyhow!("fs storage settings not found"));
    }
    Ok(settings.unwrap())
}

/// set set fs storage settings
pub async fn set(s: Settings) -> Result<()> {
    settings::set(FS_SETTINGS, &s).await?;
    Ok(())
}

/// hash hash fs storage settings
pub async fn hash() -> Result<String> {
    let settings = get().await?;
    obj_hash(settings)
}

/// new_operator load storage
pub async fn new_operator() -> Result<Operator> {
    let settings = get().await?;
    let abs_path = std::path::Path::new(&settings.local_path).canonicalize()?;
    debug!("fs storage path: {:?}", abs_path);
    std::fs::create_dir_all(&abs_path)?;
    let mut builder = Fs::default();
    builder.root(abs_path.to_str().unwrap());
    let op: Operator = Operator::new(builder)?.finish();
    Ok(op)
}
