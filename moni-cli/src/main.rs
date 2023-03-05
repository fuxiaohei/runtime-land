use clap::{Args, Parser};

mod env;

/// Command Init
#[derive(Args, Debug)]
pub struct Init {
    /// The name of the project
    pub name: String,
    /// The template to use
    #[clap(long, default_value("rust-basic"))]
    pub template: Option<String>,
}

impl Init {
    pub async fn run(&self) {
        println!("Init: {self:?}");
    }
}

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    /// Set js engine wasm file
    #[clap(long)]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) {
        println!("Build: {self:?}");
    }
}

/// Command Serve
#[derive(Args, Debug)]
pub struct Serve {
    /// The port to listen on
    #[clap(long, default_value("127.0.0.1:8668"))]
    pub addr: Option<std::net::SocketAddr>,
}

impl Serve {
    pub async fn run(&self) {
        println!("Serve: {self:?}");
    }
}

/// cli command line
#[derive(Parser)]
#[clap(name = "moni", version = moni_core::get_version())]
enum Cli {
    /// Init creates a new project
    Init(Init),
    /// Build compiles the project
    Build(Build),
    /// Serve runs the project
    Serve(Serve),
}

#[tokio::main]
async fn main() {
    moni_core::init_tracing();

    let args = Cli::parse();
    match args {
        Cli::Init(cmd) => cmd.run().await,
        Cli::Build(cmd) => cmd.run().await,
        Cli::Serve(cmd) => cmd.run().await,
    }
}
