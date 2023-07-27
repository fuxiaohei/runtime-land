use clap::Parser;
use tracing::{debug, debug_span, warn, Instrument};

mod center;
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
    #[clap(long, env("CENTER_SYNC_ENABLED"), default_value("true"))]
    pub center_sync_enabled: Option<bool>,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    localip::init().await.expect("init localip failed");

    // spawn sync internal task
    if args.center_sync_enabled.unwrap() {
        /*tokio::spawn(
            sync_interval(args.center_addr.clone(), args.center_token.clone())
                .instrument(debug_span!("[SYNC]")),
        );*/
        tokio::spawn(center::init(args.center_addr, args.center_token));
    } else {
        warn!("sync interval disabled")
    }

    server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
