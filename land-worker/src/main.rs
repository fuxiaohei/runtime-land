use anyhow::Result;
use clap::Parser;
use land_common::tracing::TraceArgs;
use land_common::version;

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
    output: TraceArgs,
    /// Token that authenticate to land-server
    #[clap(long, env = "LAND_SERVER_TOKEN", default_value = "")]
    token: String,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:9844"))]
    address: String,
    /// Data directory
    #[clap(long, env = "LAND_DATA_DIR", default_value = "./data")]
    dir: String,
    /// The url of cloud server
    #[clap(long = "url",env = "LAND_SERVER_URL", value_parser = validate_url,default_value("https://rtland-dev.zeabur.app"))]
    pub cloud_server_url: String,
    #[clap(long = "local", default_value("false"))]
    pub local_mode: bool,
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
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

    // Init ip data
    agent::ip::init().await.expect("init ip error");
    if !args.local_mode {
        agent::run_background(
            args.cloud_server_url.clone(),
            args.token.clone(),
            args.dir.clone(),
        )
        .await;
    }

    // Start the server
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
