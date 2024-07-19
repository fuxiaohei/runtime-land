use clap::Parser;
use land_common::{logging, version};
use land_core::{agent, clerk, deployer, storage};

mod admin;
mod dash;
mod server;
mod templates;
mod worker_api;

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
    /// Verbose mode.
    #[clap(flatten)]
    output: logging::TraceArgs,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:9840"))]
    address: String,
    /// Template directory
    #[clap(long)]
    tpldir: Option<String>,
    /// Database connection args.
    #[clap(flatten)]
    dbargs: land_dao::DBArgs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // Initialize tracing
    logging::init(args.output.verbose);

    // Connect to database
    land_dao::connect(&args.dbargs).await?;

    // Clerk env initialize
    clerk::init().await?;

    // Initialize background tasks
    {
        // Initialize storage
        storage::init_defaults().await?;
        storage::load_storage().await?;

        // Initialize living agent refreshing
        agent::init_livings().await;

        // Initialize handling waiting and reviewing deployments
        deployer::init_waiting().await;
    }

    // Start server
    server::start(args.address.parse()?, "./assets", args.tpldir).await?;

    Ok(())
}
