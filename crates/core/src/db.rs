use anyhow::Result;
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

#[derive(Envconfig, Debug, Serialize, Deserialize)]
pub struct DbConfig {
    #[envconfig(from = "DB_HOST", default = "localhost")]
    pub host: String,
    #[envconfig(from = "DB_PORT", default = "3306")]
    pub port: u16,
    #[envconfig(from = "DB_USER", default = "root")]
    pub user: String,
    #[envconfig(from = "DB_PASSWORD", default = "")]
    pub password: String,
    #[envconfig(from = "DB_NAME", default = "land-serverless")]
    pub database: String,
    #[envconfig(from = "DB_POOL_SIZE", default = "10")]
    pub pool_size: u32,
}

impl DbConfig {
    pub fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
    pub fn url_safe(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, "******", self.host, self.port, self.database
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
#[tracing::instrument(name="[DB]")]
pub async fn init() -> Result<()> {
    let cfg = DbConfig::init_from_env().unwrap();
    debug!("Connecting: {}", cfg.url_safe());

    let mut opt = ConnectOptions::new(cfg.url());
    opt.max_connections(cfg.pool_size)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(10))
        .max_lifetime(Duration::from_secs(10))
        .sqlx_logging(false);

    let db = Database::connect(opt).await?;
    DB.set(db).unwrap();
    info!("Init success: {}", cfg.url_safe());
    Ok(())
}
