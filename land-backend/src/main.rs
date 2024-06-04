use anyhow::Result;
use clap::Parser;
use land_common::tracing::TraceArgs;
use land_common::version;

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

    // Init clerk jwks from api
    land_service::clerk::init_envs(true).await?;

    // Block until the server stops
    tokio::signal::ctrl_c().await?;

    Ok(())
}
