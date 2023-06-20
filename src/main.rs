use clap::Parser;
use tracing::debug;

#[derive(Parser, Debug)]
#[clap(name = "land-server", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:38779"))]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("load args: {:?}", args);

    // init storage
    land_core::storage::init()
        .await
        .expect("init storage failed");

    // init db
    land_core::db::init().await.expect("init db failed");

    // init prod const
    land_core::init_prod_const()
        .await
        .expect("init prod const failed");

    // init local region
    land_core::region::local::init()
        .await
        .expect("init local region failed");

    // start restful server
    land_restful::start_server(args.http_addr.parse().unwrap())
        .await
        .unwrap();
}
