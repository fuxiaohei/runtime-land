use clap::Parser;

mod deploy;
mod embed;
mod flags;
mod server;

/// The runtime.land command line tool
#[derive(Parser)]
#[clap(name = "land-cli", version = land_core::version::get())]
#[command(about = land_core::version::get_about(), long_about = None)]
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
    land_core::trace::init();
    // console_subscriber::init();

    let args = Cli::parse();
    match args {
        Cli::Init(cmd) => cmd.run().await,
        Cli::Build(cmd) => cmd.run().await,
        Cli::Serve(cmd) => cmd.run().await,
        Cli::Deploy(cmd) => cmd.run().await,
    }
}
