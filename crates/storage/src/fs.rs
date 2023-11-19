use anyhow::Result;
use land_dao::settings;
use opendal::services::Fs;
use opendal::Operator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: "/tmp/runtime-land-data".to_string(),
        }
    }
}

impl Config {
    pub async fn save_db(&self) -> Result<()> {
        let key = settings::Key::FsStorage.to_string();
        let content = serde_json::to_string(self)?;
        let values: HashMap<String, String> = vec![(key.clone(), content)].into_iter().collect();
        settings::update_maps(values).await?;
        Ok(())
    }
}

/// create creates the local storage
pub async fn build(cfg: &Config) -> Result<Operator> {
    let mut builder = Fs::default();
    builder.root(&cfg.path);
    let op: Operator = Operator::new(builder)?.finish();
    info!("Init local storage success, path: {}", cfg.path);
    Ok(op)
}

/// reload_global reloads the global storage with the new config
pub async fn reload_global(cfg: &Config) -> Result<()> {
    let op = build(cfg).await?;
    let mut global = crate::GLOBAL.lock().await;
    *global = op;
    Ok(())
}
