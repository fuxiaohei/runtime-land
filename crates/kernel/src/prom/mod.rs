use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tracing::info;

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

/// PROM_ENV is the environment variables for Prometheus
pub static PROM_ENV: OnceCell<PromEnv> = OnceCell::new();

/// init_prom_env initializes PromEnv from environment variables
pub fn init_prom_env() -> Result<()> {
    let prom_env = PromEnv {
        addr: std::env::var("PROM_ADDR").unwrap_or_default(),
        user: std::env::var("PROM_USER").unwrap_or_default(),
        password: std::env::var("PROM_PASSWORD").unwrap_or_default(),
    };
    info!("PromEnv: {:?}", prom_env);
    PROM_ENV
        .set(prom_env)
        .map_err(|_| anyhow!("PromEnv is already set"))?;
    Ok(())
}
