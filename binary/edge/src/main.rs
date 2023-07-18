use clap::Parser;
use tracing::{debug, debug_span, Instrument};

mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-edge", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7899"))]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
