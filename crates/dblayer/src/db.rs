use anyhow::Result;
use clap::Args;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{debug, info};

#[derive(Args)]
pub struct DBArgs {
    /// Database host
    #[clap(long("db-host"), env("MYSQL_HOST"), default_value("127.0.0.1"))]
    pub host: String,
    /// Database port
    #[clap(long("db-port"), env("MYSQL_PORT"), default_value("3306"))]
    pub port: u16,
    /// Database user
    #[clap(long("db-user"), env("MYSQL_USER"), default_value("root"))]
    pub user: String,
    /// Database password
    #[clap(long("db-password"), env("MYSQL_PASSWORD"), default_value(""))]
    pub password: String,
    /// Database name
    #[clap(
        long("db-database"),
        env("MYSQL_DATABASE"),
        default_value("land-cloud-db")
    )]
    pub database: String,
    /// Database connection pool size
    #[clap(long("db-pool-size"), env("DB_POOL_SIZE"), default_value("10"))]
    pub pool_size: u32,
    /// Log SQL
    #[clap(long("db-log-sql"), env("DB_LOG_SQL"), default_value("false"))]
    pub log_sql: bool,
}

impl DBArgs {
    fn url(&self) -> String {
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
    #[tracing::instrument(skip_all, name = "[DB]")]
    pub async fn connect(&self) -> Result<()> {
        debug!("Connecting: {}", self.url_safe());

        let mut opt = ConnectOptions::new(self.url());
        opt.max_connections(self.pool_size)
            .min_connections(3)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .sqlx_logging(self.log_sql)
            .sqlx_logging_level(tracing::log::LevelFilter::Info);

        let db = Database::connect(opt).await?;

        // run migrations
        super::migration::Migrator::up(&db, None).await?;

        DB.set(db).unwrap();
        info!("Init success: {}", self.url_safe());
        Ok(())
    }
}

impl Default for DBArgs {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3306,
            user: "root".to_string(),
            password: "".to_string(),
            database: "land-cloud-db".to_string(),
            pool_size: 10,
            log_sql: false,
        }
    }
}

impl std::fmt::Debug for DBArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DBArgs")
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
