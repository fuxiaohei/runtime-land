use anyhow::Result;
use clap::Parser;
use land_common::{tracing::TraceArgs, version};

mod aliving;
mod deployer;
mod server;

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
    output: TraceArgs,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:9840"))]
    address: String,
    #[clap(flatten)]
    dbargs: land_dao::db::DBArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // Initialize tracing
    land_common::tracing::init(args.output.verbose);

    // Connect to database
    args.dbargs.connect().await?;

    // Init Defaults data in database
    land_dao::settings::init_defaults().await?;
    // Init clerk env
    land_core::auth::init_clerk_env().await?;
    // Init prometheus env
    land_core::metrics::init_prometheus().await?;

    // Start deployer background task
    deployer::run_background();

    // Start worker aliving check background task
    aliving::run_background();

    // Start the server
    server::start(args.address.parse()?, "./assets").await?;

    Ok(())
}
