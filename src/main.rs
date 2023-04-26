use clap::Parser;
use tracing::{debug, info, warn};

mod config;

#[derive(Parser, Debug)]
#[clap(name = "moni-serverless", version = moni_lib::version::get())]
struct Cli {
    /// The conf file
    #[clap(long, default_value("moni-serverless.toml"))]
    pub conf: Option<String>,
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();
    let conf_file = args.conf.as_ref().unwrap();
    // if conf_file is not exist, warn and exit
    if !std::path::Path::new(conf_file).exists() {
        warn!("conf file {} is not exist", conf_file);
        std::process::exit(1);
    }

    let conf = config::Config::from_file(conf_file).unwrap();
    debug!("load conf: {:?}", conf);

    // init db
    moni_lib::db::init(&conf.db).await.expect("init db failed");
    info!("init db success");

    // start rpc server
    moni_rpc::start_server(conf.http.addr.parse().unwrap())
        .await
        .unwrap();
}
