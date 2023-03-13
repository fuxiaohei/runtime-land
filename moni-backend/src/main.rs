use clap::Parser;
use tracing::{debug, warn, info};

mod config;

#[derive(Parser, Debug)]
#[clap(name = "moni-backend", version = moni_core::get_version())]
struct Cli {
    /// The conf file
    #[clap(long, default_value("moni-backend.toml"))]
    pub conf: Option<String>,
}

#[tokio::main]
async fn main() {
    moni_core::init_tracing();

    let args = Cli::parse();
    let conf_file = args.conf.as_ref().unwrap();
    // if conf_file is not exist, warn and exit
    if !std::path::Path::new(conf_file).exists() {
        warn!("conf file {} is not exist", conf_file);
        std::process::exit(1);
    }

    let conf = config::Config::from_file(&conf_file).unwrap();
    debug!("load conf: {:?}", conf);

    // init db pool
    moni_core::init_db(&conf.db).await.expect("init db failed");
    info!("init db success");

    // start rpc server
    moni_core::rpc::start_server(conf.http.addr.parse().unwrap())
        .await
        .unwrap();
}
