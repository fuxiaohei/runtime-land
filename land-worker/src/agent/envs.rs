use super::CLIENT;
use anyhow::{anyhow, Result};
use reqwest::header::AUTHORIZATION;
use tracing::info;

/// sync_envs syncs the envs from the server
pub async fn sync_envs(addr: String, token: String, env_md5: String) -> Result<()> {
    let url = format!("{}/api/v1/worker-api/envs", addr);
    let resp = CLIENT
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;
    let env_data: land_dao::envs::EnvWorkerLocal = resp.json().await?;
    if env_data.md5 != env_md5 {
        return Err(anyhow!("Env md5 not match"));
    }
    let mut env_local = land_dao::envs::ENV_WORKER_LOCAL.lock().await;

    let raw_data = env_data.to_raw();

    let mut env_raw_map = land_worker_server::envs::ENV.lock().await;
    *env_raw_map = raw_data;

    *env_local = env_data;
    info!("Sync envs success, md5: {}", env_md5);
    Ok(())
}
