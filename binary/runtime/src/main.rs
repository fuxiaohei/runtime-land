use anyhow::Result;
use clap::Parser;
use land_core::storage;
use tracing::{debug, debug_span, error, info, Instrument};

#[derive(Parser, Debug)]
#[clap(name = "land-endpoint", version = land_core::version::get())]
struct Cli {
    #[clap(long, env("HTTP_ADDR"), default_value("127.0.0.1:7909"))]
    pub http_addr: String,
    #[clap(long, env("STANDALONE"), default_value("false"))]
    pub standalone: bool,
    #[clap(long, env("CENTER_URL"), default_value("http://127.0.0.1:7901"))]
    pub center_url: Option<String>,
    #[clap(long, env("CENTER_TOKEN"))]
    pub center_token: Option<String>,
}

mod confs;
mod runtime;

#[tokio::main]
async fn main() -> Result<()> {
    land_core::trace::init();

    let args = Cli::parse();
    debug!("Load args: {:?}", args);

    storage::build_global("fs").await.unwrap();
    info!("Local store init success");

    if !args.standalone {
        if args.center_token.is_none() {
            error!("--center_token or CENTER_TOKEN env is required");
            Err(anyhow::anyhow!(
                "--center_token or CENTER_TOKEN env is required"
            ))?;
        }
        match confs::init(args.center_url.unwrap(), args.center_token.unwrap()).await {
            Ok(_) => info!("Confs init success"),
            Err(e) => {
                error!("Confs init failed: {:?}", e);
                Err(anyhow::anyhow!("Confs init failed: {:?}", e))?;
            }
        }
    } else {
        info!("Only runtime standalone");
    }

    runtime::start_server(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[SERVER]"))
        .await
        .unwrap();
    Ok(())
}
