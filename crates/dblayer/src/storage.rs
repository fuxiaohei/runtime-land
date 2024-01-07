use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::info;

use crate::settings::{get, set};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub current: String,
    pub fs: FsStorage,
    pub r2: R2Storage,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            current: "fs".to_string(),
            fs: FsStorage::default(),
            r2: R2Storage::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsStorage {
    pub directory: String,
}

impl Default for FsStorage {
    fn default() -> Self {
        Self {
            directory: "/tmp/runtime-land-data/".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Storage {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub base_path: String,
    pub url: Option<String>,
}

impl Default for R2Storage {
    fn default() -> Self {
        Self {
            endpoint: "http://r2.local".to_string(),
            bucket: "runtime-land".to_string(),
            region: "auto".to_string(),
            access_key: "access_key".to_string(),
            secret_key: "secret_key".to_string(),
            base_path: "runtime-land-data".to_string(),
            url: None,
        }
    }
}

pub async fn init_storage() -> Result<()> {
    let item = get("storage").await?;
    if item.is_none() {
        let value = serde_json::to_string(&Storage::default())?;
        info!("init storage: {}", value);
        set("storage", &value).await?;
    }
    Ok(())
}
