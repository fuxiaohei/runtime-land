use anyhow::Result;
use clap::Parser;
use tracing::debug;

mod apiv2;
mod confs;
mod pages;
// mod email;
mod embed;
mod region;
// mod restapi;
mod server;
mod settings;

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
    settings::init().await?;
    settings::init_storage().await?;

    // init runtime nodes
    region::init().await;

    // start confs generator loop
    confs::run(1, 30);

    crate::server::start(args.http_addr.parse().unwrap()).await?;

    Ok(())
}
