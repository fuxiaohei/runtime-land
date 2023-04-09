use clap::Parser;

mod flags;
mod  server;

/// cli command line
#[derive(Parser)]
#[clap(name = "moni-cli", version = moni_lib::version::get())]
enum Cli {
    /// Build compiles the project
    Build(flags::Build),
    /// Serve runs the project
    Serve(flags::Serve),
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();
    match args {
        Cli::Build(cmd) => cmd.run().await,
        Cli::Serve(cmd) => cmd.run().await,
    }
}
