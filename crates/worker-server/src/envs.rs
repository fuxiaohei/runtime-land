use land_dao::envs::{EnvRawData, EnvRawMap};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

/// ENV is the global variable to store envs
pub static ENV: Lazy<Mutex<EnvRawData>> = Lazy::new(|| Mutex::new(EnvRawData::new()));

/// get_by_project gets the environment variables by project uuid
pub async fn get_by_project(uuid: String) -> Option<EnvRawMap> {
    let env = ENV.lock().await;
    if let Some(project_env) = env.get(&uuid) {
        return Some(project_env.clone());
    }
    None
}
