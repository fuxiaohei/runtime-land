use clap::{CommandFactory, Parser};
use color_print::cprintln;

mod cmds;
mod embed;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    if let Err(e) = args.execute().await {
        cprintln!("<red>{}</>", e);
        std::process::exit(1);
    }
}

#[derive(Parser, Debug)]
#[clap(author, version)]
#[clap(disable_version_flag = true)] // handled manually
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = concat!(env!("CARGO_PKG_NAME")," ",env!("CARGO_PKG_VERSION")),
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
    New(cmds::New),
    Build(cmds::Build),
    Up(cmds::Up),
    Login(cmds::Login),
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
            land_common::print_version(env!("CARGO_PKG_NAME"), output.verbose);
            return Ok(());
        }

        output.init_logging();

        match output.cmd {
            Some(SubCommands::New(n)) => n.run().await,
            Some(SubCommands::Build(b)) => b.run().await,
            Some(SubCommands::Up(u)) => u.run().await,
            Some(SubCommands::Login(l)) => l.run().await,
            None => {
                CliArgs::command().print_long_help()?;
                std::process::exit(2);
            }
        }
    }
}
