use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

mod query;
pub use query::{query_range, LineSeries, MultiLineSeries, QueryRangeParams};

#[derive(Serialize, Deserialize, Clone)]
pub struct PromEnv {
    pub addr: String,
    pub user: String,
    pub password: String,
}

impl std::fmt::Debug for PromEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromEnv")
            .field("addr", &self.addr)
            .field("user", &self.user)
            .finish()
    }
}

impl Default for PromEnv {
    fn default() -> Self {
        PromEnv {
            addr: "http://localhost:9090".to_string(),
            user: "".to_string(),
            password: "".to_string(),
        }
    }
}

/// init_prometheus initializes PromEnv from environment variables
pub async fn init_prometheus() -> Result<()> {
    let item = land_dao::settings::get("prometheus").await?;
    if item.is_none() {
        let env = PromEnv::default();
        let content = serde_json::to_string(&env)?;
        land_dao::settings::set("prometheus", &content).await?;
    }
    let item = land_dao::settings::get("prometheus").await?.unwrap();
    let env: PromEnv = serde_json::from_str(item.value.as_str())?;
    info!("PromEnv initial: {:?}", env);
    Ok(())
}

/// set_env sets prometheus environment
pub async fn set_env(env: PromEnv) -> Result<()> {
    let content = serde_json::to_string(&env)?;
    land_dao::settings::set("prometheus", &content).await?;
    Ok(())
}

/// get_env gets prometheus environment
pub async fn get_env() -> Result<PromEnv> {
    let item = land_dao::settings::get("prometheus").await?.unwrap();
    let env: PromEnv = serde_json::from_str(item.value.as_str())?;
    Ok(env)
}
