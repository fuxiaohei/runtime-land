use anyhow::Result;
use clap::Parser;
use tracing::{debug, info};

mod apiv2;
mod confs;
mod pages;
// mod email;
mod embed;
// mod restapi;
mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-center", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7901"))]
    pub http_addr: String,
    #[command(flatten)]
    pub db_config: land_dao::DbConfig,
}

#[tokio::main]
#[tracing::instrument(name = "[MAIN]")]
async fn main() -> Result<()> {
    // init tracing
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    // extract embed static assets
    embed::extract_assets("static")?;

    // connect to db
    land_dao::connect(args.db_config).await?;

    // init global settings
    init_settings().await?;
    init_storage().await?;

    // start confs generator loop
    confs::run(1, 30);

    crate::server::start(args.http_addr.parse().unwrap()).await?;

    Ok(())
}

async fn init_settings() -> Result<()> {
    let (domain, protocol) = land_dao::settings::get_domain_protocol().await?;
    land_core::confdata::set_domain(domain.clone(), protocol.clone()).await;
    info!("Init, DOMAIN:{}, PROTOCOL:{}", domain, protocol);
    Ok(())
}

async fn init_storage() -> Result<()> {
    land_storage::dao::init_global_from_db().await?;
    Ok(())
}
