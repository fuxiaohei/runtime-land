use clap::Parser;
use tracing::{debug, debug_span, Instrument};

mod center;
mod conf;
mod localip;
mod server;

#[derive(Parser, Debug)]
#[clap(name = "land-edge", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7902"))]
    pub http_addr: String,
    #[clap(long, env("CENTER_URL"), default_value("http://127.0.0.1:7901"))]
    pub center_url: String,
    #[clap(long, env("CENTER_TOKEN"))]
    pub center_token: String,
}

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    localip::init().await.expect("init localip failed");

    conf::init().await.expect("init conf failed");

    tokio::spawn(center::init(args.center_url, args.center_token));

    server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
