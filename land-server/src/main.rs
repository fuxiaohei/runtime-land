use anyhow::Result;
use clap::Parser;
use land_common::{tracing::FlagArgs, version};
use land_dao::DBArgs;

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
    output: FlagArgs,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:8840"))]
    address: String,
    #[clap(flatten)]
    dbargs: DBArgs,
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

    // init defaults data
    land_dao::settings::init_defaults().await?;
    land_dao::storage::init_defatuls().await?;

    // init core background tasks
    land_core::background::init();
    // generate gateway confs in every 60 seconds
    land_core::gateway::generate_loop(1);

    // Start server
    land_server_impl::start("./assets", args.address.parse().unwrap()).await?;

    Ok(())
}
