use anyhow::Result;
use land_dao::settings;
use std::collections::HashMap;
use tracing::debug;

/// first_init_s3 init s3 storage settings to db
pub async fn first_init_s3() -> Result<()> {
    let s3_key = settings::Key::S3Storage.to_string();
    let content = serde_json::to_string(&land_storage::s3::Config::default())?;
    let values: HashMap<String, String> = vec![(s3_key.clone(), content)].into_iter().collect();
    settings::update_maps(values).await?;
    Ok(())
}

/// first_init_local init local storage settings to db
pub async fn first_init_local() -> Result<()> {
    let key = settings::Key::LocalStorage.to_string();
    let content = serde_json::to_string(&land_storage::local::Config::default())?;
    let values: HashMap<String, String> = vec![(key.clone(), content)].into_iter().collect();
    settings::update_maps(values).await?;
    Ok(())
}

async fn load_settings() -> Result<(land_storage::local::Config, land_storage::s3::Config)> {
    let s3_key = settings::Key::S3Storage.to_string();
    let local_storage_key = settings::Key::LocalStorage.to_string();
    let keys = vec![s3_key.clone(), local_storage_key.clone()];
    let settings_map = settings::list_maps(keys).await?;
    let s3_config = if let Some(s3_content) = settings_map.get(&s3_key) {
        serde_json::from_str::<land_storage::s3::Config>(s3_content)?
    } else {
        land_storage::s3::Config::default()
    };
    let local_config = if let Some(local_content) = settings_map.get(&local_storage_key) {
        serde_json::from_str::<land_storage::local::Config>(local_content)?
    } else {
        land_storage::local::Config::default()
    };
    Ok((local_config, s3_config))
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
    let (local_config, s3_config) = load_settings().await?;
    let type_name = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "local".to_string());
    match type_name.as_str() {
        "local" => {
            debug!("Init, STORAGE_TYPE:{}", "local");
            land_storage::local::reload_global(&local_config).await?;
        }
        "cloudflare-r2" => {
            debug!("Init, STORAGE_TYPE:{}", "cloudflare-r2");
            land_storage::s3::reload_global(&s3_config).await?;
        }
        _ => {
            anyhow::bail!("STORAGE_TYPE not support");
        }
    }
    update_storage_type(type_name).await?;
    Ok(())
}
