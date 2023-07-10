use clap::Parser;
use tracing::debug;

mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-center", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7777"))]
    pub http_addr: String,
    #[command(flatten)]
    pub db_config: land_dao::DbConfig,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("load args: {:?}", args);

    land_dao::connect(args.db_config).await.unwrap();

    crate::server::start(args.http_addr.parse().unwrap())
        .await
        .unwrap();
}
