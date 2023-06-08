use clap::Parser;
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[clap(name = "lol-serverless", version = lol_core::version::get())]
struct Cli {
    #[clap(long, env("MONI_GRPC_ADDR"), default_value("127.0.0.1:38779"))]
    pub grpc_addr: String,

    #[clap(long, env("MONI_GRPC_ENABLE_GRPCWEB"), default_value("true"))]
    pub enable_grpc_web: bool,
}

#[tokio::main]
async fn main() {
    lol_core::tracing::init();

    let args = Cli::parse();
    debug!("load args: {:?}", args);

    // init storage
    lol_core::storage::init()
        .await
        .expect("init storage failed");
    info!("Init storage success");

    // init db
    lol_core::db::init().await.expect("init db failed");
    info!("Init db success");

    // init prod const
    lol_core::init_prod_const()
        .await
        .expect("init prod const failed");
    info!("Init prod const success");

    // init local region
    lol_core::region::local::init()
        .await
        .expect("init local region failed");
    info!("Init local region success");

    // start rpc server
    lol_rpc::start_server(args.grpc_addr.parse().unwrap(), args.enable_grpc_web)
        .await
        .unwrap();
}
