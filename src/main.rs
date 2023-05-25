use clap::Parser;
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[clap(name = "moni-serverless", version = moni_lib::version::get())]
struct Cli {
    #[clap(long, env("MONI_GRPC_ADDR"), default_value("127.0.0.1:38779"))]
    pub grpc_addr: String,

    #[clap(long, env("MONI_GRPC_ENABLE_GRPCWEB"), default_value("true"))]
    pub enable_grpc_web: bool,
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();
    debug!("load args: {:?}", args);

    // init storage
    moni_lib::storage::init()
        .await
        .expect("init storage failed");
    info!("Init storage success");

    // init db
    moni_lib::db::init().await.expect("init db failed");
    info!("Init db success");

    // start rpc server
    moni_rpc::start_server(args.grpc_addr.parse().unwrap(), args.enable_grpc_web)
        .await
        .unwrap();
}
