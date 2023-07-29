use anyhow::Result;
use clap::Args;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{debug, info};

mod migration;
mod model;

// re-export
pub use model::project::Model as Project;
pub use model::region::Model as Region;
pub use model::settings::Model as Setting;

pub mod deployment;
pub mod project;
pub mod region;
pub mod settings;
pub mod user;
pub mod user_token;

#[derive(Args)]
pub struct DbConfig {
    #[clap(long("db-host"), env("DB_HOST"), default_value("localhost"))]
    pub host: String,
    #[clap(long("db-port"), env("DB_PORT"), default_value("3306"))]
    pub port: u16,
    #[clap(long("db-user"), env("DB_USER"), default_value("root"))]
    pub user: String,
    #[clap(long("db-password"), env("DB_PASSWORD"), default_value(""))]
    pub password: String,
    #[clap(
        long("db-database"),
        env("DB_DATABASE"),
        default_value("runtime-land-db")
    )]
    pub database: String,
    #[clap(long("db-pool-size"), env("DB_POOL_SIZE"), default_value("10"))]
    pub pool_size: u32,
    #[clap(long("db-log-sql"), env("DB_LOG_SQL"), default_value("false"))]
    pub log_sql: bool,
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
            database: "runtime-land-db".to_string(),
            pool_size: 10,
            log_sql: false,
        }
    }
}

impl std::fmt::Debug for DbConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &"******")
            .field("database", &self.database)
            .field("pool_size", &self.pool_size)
            .field("log_sql", &self.log_sql)
            .finish()
    }
}

/// DB connection pool
pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// connect to db
#[tracing::instrument(skip_all, name = "[DB]")]
pub async fn connect(cfg: DbConfig) -> Result<()> {
    debug!("Connecting: {}", cfg.url_safe());

    let mut opt = ConnectOptions::new(cfg.url());
    opt.max_connections(cfg.pool_size)
        .min_connections(3)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(cfg.log_sql);

    let db = Database::connect(opt).await?;

    // run migrations
    migration::Migrator::up(&db, None).await?;

    DB.set(db).unwrap();
    info!("Init success: {}", cfg.url_safe());
    Ok(())
}
