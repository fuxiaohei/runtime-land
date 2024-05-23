use clap::{CommandFactory, Parser};
use color_print::cprintln;
use land_common::tracing::TraceArgs;
use land_common::version;
use std::process;

mod cmds;
mod embed;

#[derive(Parser, Debug)]
enum SubCommands {
    New(cmds::New),
    Build(cmds::Build),
    Up(cmds::Up),
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
    output: TraceArgs,
    #[clap(subcommand)]
    cmd: Option<SubCommands>,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return;
    }
    land_common::tracing::init(args.output.verbose);

    let res = match args.cmd {
        Some(SubCommands::New(n)) => n.run().await,
        Some(SubCommands::Build(b)) => b.run().await,
        Some(SubCommands::Up(u)) => u.run().await,
        None => {
            CliArgs::command().print_long_help().unwrap();
            process::exit(2);
        }
    };
    if let Err(err) = res {
        cprintln!("<red>Something wrong:\n  {}</red>", err);
        process::exit(2);
    }
}
