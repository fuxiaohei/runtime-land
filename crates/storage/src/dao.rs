use crate::{FsConfig, S3Config};
use anyhow::Result;
use land_dao::settings;

/// load loads the storage settings from db
pub async fn load() -> Result<(String, FsConfig, S3Config)> {
    let type_key = settings::Key::StorageType.to_string();
    let s3_key = settings::Key::S3Storage.to_string();
    let fs_storage_key = settings::Key::FsStorage.to_string();
    let keys = vec![type_key.clone(), s3_key.clone(), fs_storage_key.clone()];

    let settings_map = settings::list_maps(keys).await?;

    let s3_config = if let Some(s3_content) = settings_map.get(&s3_key) {
        serde_json::from_str::<S3Config>(s3_content)?
    } else {
        S3Config::default()
    };

    let fs_config = if let Some(local_content) = settings_map.get(&fs_storage_key) {
        serde_json::from_str::<FsConfig>(local_content)?
    } else {
        FsConfig::default()
    };

    let default_value = "fs".to_string();
    let type_key_value = settings_map.get(&type_key).unwrap_or(&default_value);
    Ok((type_key_value.to_string(), fs_config, s3_config))
}
