use crate::models::settings;
use crate::DB;
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::info;

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

pub async fn init() -> Result<()> {
    let item = get("storage").await?;
    if item.is_none() {
        let value = serde_json::to_string(&Storage::default())?;
        info!("init storage: {}", value);
        set("storage", &value).await?;
    }
    Ok(())
}

pub async fn get(name: &str) -> Result<Option<settings::Model>> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(item)
}

pub async fn set(name: &str, value: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let now = chrono::Utc::now();
    if item.is_none() {
        let value = serde_json::to_string(&Storage::default())?;
        let item = settings::ActiveModel {
            id: Set(0),
            name: Set(name.to_string()),
            label: Set(name.to_string()),
            value: Set(value),
            created_at: Set(now),
            updated_at: Set(now),
        };
        item.insert(db).await?;
    } else {
        let item = item.unwrap();
        let mut item = item.into_active_model();
        item.value = Set(value.to_string());
        item.updated_at = Set(now);
        item.save(db).await?;
    }
    Ok(())
}
