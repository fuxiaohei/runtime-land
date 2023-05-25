use clap::Parser;
use tracing::{debug, debug_span, Instrument};

pub mod rt_server;

#[derive(Parser, Debug)]
#[clap(name = "moni-runtime", version = moni_lib::version::get())]
struct Cli {
    #[clap(long, env("MONI_HTTP_ADDR"), default_value("127.0.0.1:38889"))]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();

    debug!("load args: {:?}", args);

    rt_server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[Http]"))
        .await
        .unwrap();
}
