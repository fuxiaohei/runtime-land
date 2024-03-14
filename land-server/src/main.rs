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

    // start crons
    land_kernel::cron::start(land_kernel::cron::Options {
        gen_deploys: 1,
        review_tasks: 1,
    });
    land_kernel::tasks::init().await?;

    // Start server
    land_server_impl::start("./assets", args.address.parse().unwrap()).await?;

    Ok(())
}
