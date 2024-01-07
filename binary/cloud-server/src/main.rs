use anyhow::Result;
use axum::{routing::any, Router};
use clap::Parser;
use color_print::cprintln;
use land_dblayer::DBArgs;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser, Debug)]
struct OutputArgs {
    /// Generate verbose output
    #[clap(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,
    /// Do not print progress messages.
    #[clap(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
}

impl OutputArgs {
    pub fn init_logging(&self) {
        land_common::init_logging(self.verbose);
    }
}

#[derive(Parser, Debug)]
#[clap(author, version)]
#[clap(disable_version_flag = true)] // handled manually
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = concat!(env!("CARGO_PKG_NAME")," ",env!("CARGO_PKG_VERSION")),
)]
struct Args {
    /// Print version info and exit.
    #[clap(short = 'V', long)]
    version: bool,
    #[clap(flatten)]
    output: OutputArgs,
    /// Address to listen on.
    #[clap(long, default_value("127.0.0.1:3040"))]
    address: String,
    #[clap(flatten)]
    db_args: DBArgs,
}

impl Args {
    async fn exeucte(self) -> Result<()> {
        let Args {
            version,
            output,
            address,
            db_args,
        } = self;
        if version {
            land_common::print_version(env!("CARGO_PKG_NAME"), output.verbose);
            return Ok(());
        }
        output.init_logging();

        // connect db
        db_args.connect().await?;

        // init preset data
        land_dblayer::settings::init_settings().await?;
        land_dblayer::storage::init_storage().await?;

        // extract assets
        let assets_dir = "assets";
        land_web_server::extract_assets(assets_dir)?;

        // merge router api and website api
        let router = Router::new()
            .route("/", any(land_web_server::default_handler))
            .merge(land_web_server::router(assets_dir)?)
            .merge(land_api_server::router());

        start_server(address.parse().unwrap(), router).await?;

        Ok(())
    }
}

pub async fn start_server(addr: SocketAddr, app: Router) -> Result<()> {
    info!("Starting on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(e) = args.exeucte().await {
        cprintln!("<red>Error:</> {}", e);
        std::process::exit(1);
    }
}
