use clap::Parser;
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[clap(name = "land-server", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("GRPC_ADDR"), default_value("127.0.0.1:38779"))]
    pub grpc_addr: String,

    #[clap(long, env("GRPC_ENABLE_GRPCWEB"), default_value("true"))]
    pub enable_grpc_web: bool,
}

#[tokio::main]
async fn main() {
    land_core::tracing::init();

    let args = Cli::parse();
    debug!("load args: {:?}", args);

    // init storage
    land_core::storage::init()
        .await
        .expect("init storage failed");
    info!("Init storage success");

    // init db
    land_core::db::init().await.expect("init db failed");
    info!("Init db success");

    // init prod const
    land_core::init_prod_const()
        .await
        .expect("init prod const failed");
    info!("Init prod const success");

    // init local region
    land_core::region::local::init()
        .await
        .expect("init local region failed");
    info!("Init local region success");

    // start rpc server
    land_rpc::start_server(args.grpc_addr.parse().unwrap(), args.enable_grpc_web)
        .await
        .unwrap();
}
