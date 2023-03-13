use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

#[derive(Debug, Serialize, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
}

impl DbConfig {
    pub fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3306,
            user: "root".to_string(),
            password: "".to_string(),
            database: "moss-serverless".to_string(),
            pool_size: 10,
        }
    }
}

/// DB connection pool
pub static DB: OnceCell<MySqlPool> = OnceCell::new();

/// init_db initializes database connection pool
pub async fn init_db(cfg: &DbConfig) -> Result<()> {
    let pool = MySqlPoolOptions::new()
        .max_connections(cfg.pool_size)
        .connect(&cfg.url())
        .await?;
    DB.set(pool).unwrap();
    Ok(())
}
