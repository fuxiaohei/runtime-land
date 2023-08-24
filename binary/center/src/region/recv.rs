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

    let (typename, _local_config, s3_config) = load_storage_settings().await?;
    if typename == "s3" {
        send_data.storage_basepath = s3_config.bucket_basepath;
    }
    Ok(send_data)
}
