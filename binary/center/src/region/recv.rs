use super::conf::CONF_VALUES;
use crate::settings::load_storage_settings;
use anyhow::Result;
use land_core::confdata::RegionRecvData;

pub async fn build_data() -> Result<RegionRecvData> {
    let conf_values = CONF_VALUES.lock().await.clone();
    let mut send_data = RegionRecvData {
        conf_values,
        storage_basepath: String::new(),
    };

    let (type_key_value, _local_config, s3_config) = load_storage_settings().await?;
    let type_value = land_storage::Type::from_str(&type_key_value.as_str());
    if type_value == land_storage::Type::CloudflareR2 {
        send_data.storage_basepath = s3_config.bucket_basepath;
    }
    Ok(send_data)
}
