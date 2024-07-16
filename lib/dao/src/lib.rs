use anyhow::Result;
use clap::Args;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{debug, info};

mod migration;
pub mod models;

#[derive(Args)]
pub struct DBArgs {
    /// Database host
    #[clap(long("db-host"), env("POSTGRES_HOST"), default_value("127.0.0.1"))]
    pub host: String,
    /// Database port
    #[clap(long("db-port"), env("POSTGRES_PORT"), default_value("5432"))]
    pub port: u16,
    /// Database user
    #[clap(long("db-user"), env("POSTGRES_USER"), default_value("fuxiaohei"))]
    pub user: String,
    /// Database password
    #[clap(
        long("db-password"),
        env("POSTGRES_PASSWORD"),
        default_value("fuxiaohei")
    )]
    pub password: String,
    /// Database name
    #[clap(
        long("db-database"),
        env("POSTGRES_DATABASE"),
        default_value("runtime-land-db")
    )]
    pub database: String,
    /// Database connection pool size
    #[clap(long("db-pool-size"), env("DB_POOL_SIZE"), default_value("10"))]
    pub pool_size: u32,
}

impl DBArgs {
    fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
    pub fn url_safe(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, "******", self.host, self.port, self.database
        )
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
            .finish()
    }
}

/// DB connection pool
pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// connect connects to the database.
pub async fn connect(args: &DBArgs) -> Result<()> {
    debug!("DB Connecting: {}", args.url_safe());

    let mut opt = ConnectOptions::new(args.url());
    opt.max_connections(args.pool_size)
        .min_connections(3)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    let db = Database::connect(opt).await?;

    // run migrations
    migration::Migrator::up(&db, None).await?;

    DB.set(db).unwrap();
    info!("DB Init success: {}", args.url_safe());

    // check installed
    /*
    if settings::check_installed().await? {
        info!("Already installed");
        settings::load_defaults().await?;
    } else {
        warn!("Not installed");
    }*/

    Ok(())
}

fn _now_time() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

/// DateTimeUTC is a UTC time type, for easy to use in dao models
pub type DateTimeUTC = chrono::DateTime<chrono::Utc>;
