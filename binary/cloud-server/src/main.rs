use anyhow::Result;
use clap::Parser;
use color_print::cprintln;
use land_dblayer::DBArgs;

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

        // start api server
        land_api_server::run(address.parse().unwrap()).await?;

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
