use anyhow::Result;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

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
pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// init initializes database connection pool
pub async fn init(cfg: &DbConfig) -> Result<()> {
    let url = cfg.url();
    debug!("connect to database: {url}");

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(cfg.pool_size)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(10))
        .max_lifetime(Duration::from_secs(10))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    DB.set(db).unwrap();
    Ok(())
}
