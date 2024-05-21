use anyhow::Result;
use land_dao::envs::{EnvWorkerItem, EnvWorkerTotal, ENV_WORKER_LOCAL};
use std::collections::HashMap;
use tracing::{info, instrument};

#[instrument("[Envs::Refresh]")]
pub async fn refresh() -> Result<()> {
    let envs = land_dao::envs::list_envs().await?;
    //debug!("Found {} envs", envs.len());
    let mut total: EnvWorkerTotal = HashMap::new();
    for env in envs {
        let item = EnvWorkerItem {
            key: env.env_key,
            value: env.env_value,
            salt: env.env_salt,
        };
        total.entry(env.project_uuid).or_default().push(item);
    }
    let total_content = serde_json::to_string(&total)?;
    let total_md5 = format!("{:x}", md5::compute(total_content.as_bytes()));

    let mut local = ENV_WORKER_LOCAL.lock().await;
    if local.md5 == total_md5 {
        //debug!("No need to update envs");
        return Ok(());
    }
    local.md5 = total_md5;
    local.envs = total;
    info!("Updated envs, md5: {}", local.md5);
    Ok(())
}
