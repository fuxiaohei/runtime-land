use anyhow::{anyhow, Result};
use clap::Parser;
use land_common::{tracing::FlagArgs, version};
use land_core::workerinfo::sync;

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
    #[clap(long = "url",env = "LAND_SERVER_URL", value_parser = validate_url,default_value("https://cc.runtime.land"))]
    pub cloud_server_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }
    land_common::tracing::init(args.output.verbose);
    if args.token.is_empty() {
        return Err(anyhow!("LAND_SERVER_TOKEN is required"));
    }

    // get local ip data
    land_core::ip::init().await?;
    // sync confs loop
    let target_file = format!("{}/traefik.yaml", args.dir);
    let opts = sync::Opts {
        cloud_server_addr: args.cloud_server_url.clone(),
        token: args.token.clone(),
        data_dir: args.dir.clone(),
        conf_file: target_file.clone(),
        server_addr: args.address.clone(),
    };
    sync::run_loop(1, opts);

    let opts = land_worker_server::Opts {
        addr: args.address.parse()?,
        dir: args.dir.clone(),
        default_wasm: "".to_string(),
        endpoint_name: None,
        wasm_aot: true,
    };
    land_worker_server::start(opts).await?;
    Ok(())
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}
