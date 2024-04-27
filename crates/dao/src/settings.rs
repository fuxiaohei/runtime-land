use crate::{db::DB, models::settings, now_time};
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use opendal::services::{Fs, Memory, S3};
use opendal::Operator;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::info;

static DOMAIN_SETTINGS: &str = "domain_settings";

#[derive(Serialize, Deserialize)]
pub struct DomainSettings {
    pub domain: String,
    pub protocol: String,
}

/// init_defaults initializes default settings
pub async fn init_defaults() -> Result<()> {
    init_default_domains().await?;
    init_default_storage().await?;
    Ok(())
}

async fn init_default_domains() -> Result<()> {
    let item = get(DOMAIN_SETTINGS).await?;
    if item.is_none() {
        let content = serde_json::to_string(&DomainSettings {
            domain: "127-0-0-1.nip.io".to_string(),
            protocol: "http".to_string(),
        })?;
        set(DOMAIN_SETTINGS, &content).await?;
        info!("Init domain settings: {}", content);
    }
    Ok(())
}

/// get_domain_settings returns domain suffix and protocol
pub async fn get_domain_settings() -> Result<(String, String)> {
    let content = get(DOMAIN_SETTINGS).await?.unwrap().value;
    let settings: DomainSettings = serde_json::from_str(content.as_str())?;
    Ok((settings.domain, settings.protocol))
}

/// set_domain_settings sets domain suffix and protocol
pub async fn set_domain_settings(domain: String, protocol: String) -> Result<()> {
    let content = serde_json::to_string(&DomainSettings { domain, protocol })?;
    set(DOMAIN_SETTINGS, &content).await?;
    Ok(())
}

/// list_names returns all settings names
pub async fn list_names() -> Result<Vec<String>> {
    let db = DB.get().unwrap();
    let items = settings::Entity::find()
        .order_by_asc(settings::Column::Name)
        .all(db)
        .await?;
    let names = items.iter().map(|item| item.name.clone()).collect();
    Ok(names)
}

/// get settings item
pub async fn get(name: &str) -> Result<Option<settings::Model>> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(item)
}

/// set settings item
pub async fn set(name: &str, value: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let now = now_time();
    if item.is_none() {
        let item = settings::ActiveModel {
            name: Set(name.to_string()),
            value: Set(value.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
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

impl Storage {
    pub fn build_url(&self, path: &str) -> Result<String> {
        if self.current == "fs" {
            return Ok(format!("file://{}", path));
        }
        if self.current == "r2" {
            if !self.r2.base_path.is_empty() {
                return Ok(format!(
                    "{}/{}/{}",
                    self.r2.url.as_ref().unwrap(),
                    self.r2.base_path,
                    path
                ));
            }
            return Ok(format!("{}/{}", self.r2.url.as_ref().unwrap(), path));
        }
        Err(anyhow::anyhow!("Unknown storage: {}", self.current))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsStorage {
    pub directory: String,
}

impl Default for FsStorage {
    fn default() -> Self {
        Self {
            directory: "/data/runtime-land-data/".to_string(),
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

// init_default_storage initializes default storage settings
async fn init_default_storage() -> Result<()> {
    let item = get("storage").await?;
    if item.is_none() {
        let value = serde_json::to_string(&Storage::default())?;
        info!("Init storage defaults: {}", value);
        set("storage", &value).await?;
    }
    reload_storage().await?;
    Ok(())
}

/// STORAGE is the global storage operator
pub static STORAGE: Lazy<Mutex<Operator>> = Lazy::new(|| {
    let mut builder = Memory::default();
    builder.root("/tmp");
    let op = Operator::new(builder).unwrap().finish();
    Mutex::new(op)
});

/// reload_storage reloads storage from database to GLOBAL
pub async fn reload_storage() -> Result<()> {
    let item = get("storage").await?;
    let storage: Storage = serde_json::from_str(&item.unwrap().value)?;

    // if storage is fs
    if storage.current == "fs" {
        let mut builder = Fs::default();
        builder.root(&storage.fs.directory);
        let op = Operator::new(builder).unwrap().finish();
        let mut global = STORAGE.lock().await;
        *global = op;
        info!("Build global storage: fs, path:{:?}", storage.fs.directory);
        return Ok(());
    }

    // if storage is r2
    if storage.current == "r2" {
        let mut builder = S3::default();
        builder.root(&storage.r2.base_path);
        builder.bucket(&storage.r2.bucket);
        builder.endpoint(&storage.r2.endpoint);
        builder.region(&storage.r2.region);
        builder.batch_max_operations(300); // cloudflare R2 need < 700
        builder.access_key_id(&storage.r2.access_key);
        builder.secret_access_key(&storage.r2.secret_key);

        let op: Operator = Operator::new(builder)?.finish();
        let mut global = STORAGE.lock().await;
        *global = op;
        info!(
            "Build global storage: r2, endpoint:{:?}",
            storage.r2.endpoint
        );
        return Ok(());
    }

    // unknown storage
    Err(anyhow::anyhow!("Unknown storage: {}", storage.current))
}

/// get_storage returns the current storage
pub async fn get_storage() -> Result<Storage> {
    let item = get("storage").await?;
    let storage: Storage = serde_json::from_str(&item.unwrap().value)?;
    Ok(storage)
}
