use clap::Parser;
use tracing::{debug, debug_span, Instrument};

mod pool;
mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-runtime", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7888"))]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();

    // init storage
    land_storage::init().await.expect("init storage failed");

    server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();

    debug!("Load args: {:?}", args);
}
