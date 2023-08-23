use anyhow::Result;
use envconfig::Envconfig;
use land_dao::settings;
use land_storage::{FsConfig, S3Config};
use std::collections::HashMap;
use tracing::{debug, error, warn};

/// install_s3 init s3 storage settings to db
async fn install_s3() -> Result<()> {
    let cfg = S3Config::default();
    warn!("Install S3 storage with empty config, please update it later in dashboard admin panel");
    update_s3(&cfg).await
}

/// install_fs init fs storage settings to db
async fn install_fs() -> Result<()> {
    let key = settings::Key::LocalStorage.to_string();
    let cfg = FsConfig::init_from_env()?;
    let content = serde_json::to_string(&cfg)?;
    let values: HashMap<String, String> = vec![(key.clone(), content)].into_iter().collect();
    settings::update_maps(values).await?;
    warn!("Install fs storage success, path: {}", cfg.path);
    Ok(())
}

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
    let default_value = "unknown".to_string();
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

    // if current_type is unknown, use env STORAGE_TYPE to install storage data first
    if current_type == "unknown" {
        return install_storage().await;
    }
    let type_name = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| current_type.clone());
    let type_value = land_storage::Type::from_str(&type_name);
    match type_value {
        land_storage::Type::Fs => {
            debug!("Init, STORAGE_TYPE:{}", "fs");
            land_storage::reload_fs_global(&local_config).await?;
        }
        land_storage::Type::CloudflareR2 => {
            debug!("Init, STORAGE_TYPE:{}", "cloudflare-r2");
            land_storage::reload_s3_global(&s3_config).await?;
        }
        land_storage::Type::Unknown => {
            anyhow::bail!("STORAGE_TYPE not support");
        }
    }
    update_storage_type(type_name).await?;
    Ok(())
}

async fn install_storage() -> Result<()> {
    let type_name = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| String::new());
    if type_name.is_empty() {
        error!("STORAGE_TYPE not set");
        anyhow::bail!("STORAGE_TYPE not set");
    }
    let type_value = land_storage::Type::from_str(&type_name);
    match type_value {
        land_storage::Type::Fs => {
            debug!("Init, STORAGE_TYPE:{}", "fs");
            install_fs().await?;
        }
        land_storage::Type::CloudflareR2 => {
            debug!("Init, STORAGE_TYPE:{}", "cloudflare-r2");
            install_s3().await?;
        }
        land_storage::Type::Unknown => {
            error!("STORAGE_TYPE {} not support", type_name);
            anyhow::bail!("STORAGE_TYPE not support");
        }
    }
    update_storage_type(type_name.clone()).await?;
    warn!("Install storage success, STORAGE_TYPE:{}", type_name);
    Ok(())
}

async fn update_s3(cfg: &S3Config) -> Result<()> {
    let s3_key = settings::Key::S3Storage.to_string();
    let content = serde_json::to_string(cfg)?;
    let values: HashMap<String, String> = vec![(s3_key.clone(), content)].into_iter().collect();
    settings::update_maps(values).await?;
    Ok(())
}

/// reload_s3 reload s3 storage settings to db
pub async fn reload_s3(cfg: &S3Config) -> Result<()> {
    update_s3(cfg).await?;
    land_storage::reload_s3_global(cfg).await?;
    Ok(())
}
