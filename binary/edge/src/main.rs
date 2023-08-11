use clap::Parser;
use tracing::{debug, debug_span, Instrument};

mod center;
mod conf;
mod localip;
mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-edge", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7899"))]
    pub http_addr: String,
    #[clap(long, env("CENTER_ADDR"), default_value("127.0.0.1:7777"))]
    pub center_addr: String,
    #[clap(long, env("CENTER_TOKEN"))]
    pub center_token: String,
    #[clap(long, env("CENTER_PROTOCOL"), default_value("ws"))]
    pub center_protocol: String,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    localip::init().await.expect("init localip failed");

    conf::init().await.expect("init conf failed");

    tokio::spawn(center::init(
        args.center_addr,
        args.center_protocol,
        args.center_token,
    ));

    server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
