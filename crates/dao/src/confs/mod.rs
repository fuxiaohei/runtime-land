use crate::models::deployment::Model as DeploymentModel;
use crate::settings::Storage;
use anyhow::Result;
use serde::{Deserialize, Serialize};

mod traefik;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskValue {
    pub user_uuid: String,
    pub project_uuid: String,
    pub domain: String,
    pub download_url: String,
    pub wasm_path: String,
    pub task_id: String,
    pub checksum: String,
    pub traefik: Option<String>,
    pub traefik_checksum: Option<String>,
}

impl TaskValue {
    pub fn new(
        dp: &DeploymentModel,
        s: &Storage,
        domain: &str,
        service_name: &str,
    ) -> Result<Self> {
        let mut task_value = TaskValue {
            user_uuid: dp.user_uuid.clone(),
            project_uuid: dp.project_uuid.clone(),
            domain: format!("{}.{}", dp.domain, domain),
            download_url: s.build_url(&dp.storage_path)?,
            wasm_path: dp.storage_path.clone(),
            task_id: dp.task_id.clone(),
            checksum: dp.storage_md5.clone(),
            traefik: None,
            traefik_checksum: None,
        };
        let traefik_conf = traefik::build_item(&task_value,service_name)?;
        let traefik_content = serde_yaml::to_string(&traefik_conf)?;
        let traefik_checksum = format!("{:x}", md5::compute(traefik_content.as_bytes()));
        task_value.traefik = Some(traefik_content);
        task_value.traefik_checksum = Some(traefik_checksum);
        Ok(task_value)
    }
}
