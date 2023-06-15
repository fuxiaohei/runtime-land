use clap::Parser;
use land_core::{storage, version};
use tracing::{debug, debug_span, Instrument};

#[derive(Parser, Debug)]
#[clap(name = "land-runtime", version = version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:38889"))]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();

    debug!("load args: {:?}", args);

    // init storage
    storage::init().await.expect("init storage failed");

    land_runtime::server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
