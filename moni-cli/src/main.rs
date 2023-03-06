use clap::Parser;

mod embed;
mod env;
mod flags;
mod server;

/// cli command line
#[derive(Parser)]
#[clap(name = "moni", version = moni_core::get_version())]
enum Cli {
    /// Init creates a new project
    Init(flags::Init),
    /// Build compiles the project
    Build(flags::Build),
    /// Serve runs the project
    Serve(flags::Serve),
    /// Login to cloud server
    Login(flags::Login),
    /// Deploy to cloud server
    Deploy(flags::Deploy),
}

#[tokio::main]
async fn main() {
    moni_core::init_tracing();

    let args = Cli::parse();
    match args {
        Cli::Init(cmd) => cmd.run().await,
        Cli::Build(cmd) => cmd.run().await,
        Cli::Serve(cmd) => cmd.run().await,
        Cli::Login(cmd) => cmd.run().await,
        Cli::Deploy(cmd) => cmd.run().await,
    }
}
