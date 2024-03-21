use anyhow::{anyhow, Result};
use clap::Parser;
use land_common::{tracing::FlagArgs, version};

mod agent;

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
    /// Token that authenticate to land-server
    #[clap(long, env = "LAND_SERVER_TOKEN", default_value = "")]
    token: String,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:8866"))]
    address: String,
    /// Data directory
    #[clap(long, env = "LAND_DATA_DIR", default_value = "./data/land")]
    dir: String,
    /// The url of cloud server
    #[clap(long = "url",env = "LAND_SERVER_URL", value_parser = validate_url,default_value("https://rtland-dev.zeabur.app"))]
    pub cloud_server_url: String,
    #[clap(long = "local", default_value("false"))]
    pub local_mode: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }
    land_common::tracing::init(args.output.verbose);
    if args.token.is_empty() && !args.local_mode {
        return Err(anyhow!("LAND_SERVER_TOKEN is required"));
    }

    // get local ip data
    agent::ip::init().await?;
    // local mode do not get data from center
    if !args.local_mode {
        // run worker-agent role
        agent::run(args.cloud_server_url, args.token, args.dir.clone()).await?;
    }

    let opts = land_worker_server::Opts {
        addr: args.address.parse()?,
        dir: args.dir.clone(),
        default_wasm: "".to_string(),
        endpoint_name: None,
        wasm_aot: true,
        metrics: true,
    };
    land_worker_server::start(opts).await?;
    Ok(())
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}
