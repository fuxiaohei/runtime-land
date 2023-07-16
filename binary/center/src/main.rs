use anyhow::Result;
use clap::Parser;
use tracing::{debug, info};

mod server;
mod restapi;

#[derive(Parser, Debug)]
#[clap(name = "land-center", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7777"))]
    pub http_addr: String,
    #[command(flatten)]
    pub db_config: land_dao::DbConfig,
}

#[tokio::main]
#[tracing::instrument(name = "[MAIN]")]
async fn main() -> Result<()> {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    land_dao::connect(args.db_config).await?;
    info!("Connect to database success");

    crate::server::start(args.http_addr.parse().unwrap()).await?;

    Ok(())
}
