use anyhow::Result;
use clap::Parser;
use color_print::cprintln;
use land_worker_server::Opts;

mod confs;

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
    /// The url of cloud server
    #[clap(long = "url", value_parser = validate_url,default_value("https://cloud.runtime.land"))]
    pub cloud_server_url: Option<String>,
    /// The token of the runner
    #[clap(long)]
    pub token: String,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:3050"))]
    pub address: String,
    /// confs server Address to listen on.
    #[clap(long, default_value("0.0.0.0:3051"))]
    pub confs_address: String,
}

impl Args {
    async fn exeucte(self) -> Result<()> {
        let Args {
            version,
            output,
            cloud_server_url,
            token,
            address,
            confs_address,
        } = self;
        if version {
            land_common::print_version(env!("CARGO_PKG_NAME"), output.verbose);
            return Ok(());
        }
        output.init_logging();

        // init confs loop
        confs::init_loop(token, cloud_server_url.unwrap(), address.clone())?;

        // start confs server
        tokio::spawn(async move {
            confs::start_server(confs_address.parse().unwrap())
                .await
                .unwrap();
        });

        let opts = Opts {
            addr: address.parse().unwrap(),
            ..Default::default()
        };
        land_worker_server::run(opts).await.unwrap();

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(e) = args.exeucte().await {
        cprintln!("<red>Error:</> {}", e);
        std::process::exit(1);
    }
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}
