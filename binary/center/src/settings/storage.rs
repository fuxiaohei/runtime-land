use anyhow::Result;
use land_dao::settings;
use land_storage::{FsConfig, S3Config};
use std::collections::HashMap;
use tracing::debug;

pub async fn load_settings() -> Result<(String, FsConfig, S3Config)> {
    let type_key = settings::Key::StorageType.to_string();
    let s3_key = settings::Key::S3Storage.to_string();
    let local_storage_key = settings::Key::LocalStorage.to_string();
    let keys = vec![type_key.clone(), s3_key.clone(), local_storage_key.clone()];
    let settings_map = settings::list_maps(keys).await?;
    let s3_config = if let Some(s3_content) = settings_map.get(&s3_key) {
        serde_json::from_str::<S3Config>(s3_content)?
    } else {
        S3Config::default()
    };
    let local_config = if let Some(local_content) = settings_map.get(&local_storage_key) {
        serde_json::from_str::<FsConfig>(local_content)?
    } else {
        FsConfig::default()
    };
    let default_value = "fs".to_string();
    let type_key_value = settings_map.get(&type_key).unwrap_or(&default_value);
    Ok((type_key_value.to_string(), local_config, s3_config))
}

/// update_storage_type update storage type
async fn update_storage_type(stype: String) -> Result<()> {
    let key = settings::Key::StorageType.to_string();
    let values: HashMap<String, String> = vec![(key.clone(), stype)].into_iter().collect();
    settings::update_maps(values).await?;
    Ok(())
}

/// init storage
#[tracing::instrument(name = "[STORAGE]")]
pub async fn init() -> Result<()> {
    let (current_type, local_config, s3_config) = load_settings().await?;
    match current_type.as_str() {
        "fs" => {
            debug!("Init, STORAGE_TYPE:{}", "fs");
            land_storage::reload_fs_global(&local_config).await?;
        }
        "s3" => {
            debug!("Init, STORAGE_TYPE:{}", "s3");
            land_storage::reload_s3_global(&s3_config).await?;
        }
        _ => {
            anyhow::bail!("STORAGE_TYPE not support");
        }
    }
    Ok(())
}

/// reload_s3 reload s3 storage settings to db
pub async fn reload_s3(cfg: &S3Config) -> Result<()> {
    let s3_key = settings::Key::S3Storage.to_string();
    let content = serde_json::to_string(cfg)?;
    let values: HashMap<String, String> = vec![(s3_key.clone(), content)].into_iter().collect();
    settings::update_maps(values).await?;
    land_storage::reload_s3_global(cfg).await?;
    update_storage_type("s3".to_string()).await?;
    Ok(())
}

/// reload_s3 reload filesystem storage settings to db
pub async fn reload_fs(cfg: &FsConfig) -> Result<()> {
    let local_storage_key = settings::Key::LocalStorage.to_string();
    let content = serde_json::to_string(cfg)?;
    let values: HashMap<String, String> = vec![(local_storage_key.clone(), content)]
        .into_iter()
        .collect();
    settings::update_maps(values).await?;
    land_storage::reload_fs_global(cfg).await?;
    update_storage_type("fs".to_string()).await?;
    Ok(())
}
