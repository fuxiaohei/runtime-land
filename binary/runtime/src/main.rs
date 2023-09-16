use clap::Parser;
use tracing::{debug, debug_span, Instrument};

#[derive(Parser, Debug)]
#[clap(name = "land-endpoint", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7909"))]
    pub http_addr: String,
    #[clap(long, env("CENTER_URL"), default_value("http://127.0.0.1:7901"))]
    pub center_url: String,
    #[clap(long, env("CENTER_TOKEN"))]
    pub center_token: String,
}

mod confs;
mod runtime;

#[tokio::main]
async fn main() {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    confs::init(args.center_url, args.center_token).await;

    runtime::start_server(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
}
