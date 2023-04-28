use clap::Parser;

mod embed;
mod flags;
mod server;

/// cli command line
#[derive(Parser)]
#[clap(name = "moni-cli", version = moni_lib::version::get())]
enum Cli {
    /// Init creates a new project
    Init(flags::Init),
    /// Build compiles the project
    Build(flags::Build),
    /// Serve runs the project
    Serve(flags::Serve),
    /// Deploy to cloud server
    Deploy(flags::Deploy),
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();
    match args {
        Cli::Init(cmd) => cmd.run().await,
        Cli::Build(cmd) => cmd.run().await,
        Cli::Serve(cmd) => cmd.run().await,
        Cli::Deploy(cmd) => cmd.run().await,
    }
}
