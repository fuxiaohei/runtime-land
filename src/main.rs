use clap::{CommandFactory, Parser};

mod commands;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    if let Err(e) = args.execute().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[derive(Parser, Debug)]
#[clap(author, version)]
#[clap(disable_version_flag = true)] // handled manually
#[clap(
    name = "land-cli",
    about = concat!("land-cli ", env!("CARGO_PKG_VERSION")),
)]
struct CliArgs {
    /// Print version info and exit.
    #[clap(short = 'V', long)]
    version: bool,
    #[clap(flatten)]
    output: OutputArgs,
}

#[derive(Parser, Debug)]
enum SubCommands {
    Init(commands::Init),
}

#[derive(Parser, Debug)]
struct OutputArgs {
    /// Generate verbose output
    #[clap(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,
    /// Do not print progress messages.
    #[clap(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    #[clap(subcommand)]
    cmd: Option<SubCommands>,
}

impl OutputArgs {
    pub fn init_logging(&self) {
        land_common::init_logging(self.verbose);
    }
}

impl CliArgs {
    async fn execute(self) -> Result<(), anyhow::Error> {
        let CliArgs { version, output } = self;
        if version {
            land_common::print_version(env!("CARGO_CRATE_NAME"), output.verbose);
            return Ok(());
        }

        output.init_logging();

        match output.cmd {
            Some(SubCommands::Init(init)) => init.run().await,
            None => {
                CliArgs::command().print_long_help()?;
                std::process::exit(2);
            }
        }
    }
}
