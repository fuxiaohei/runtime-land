use clap::Args;
use tracing::debug;

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    /// Set js engine wasm file
    #[clap(long)]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) {
        debug!("Build: {self:?}");
    }
}

/// Command Serve
#[derive(Args, Debug)]
pub struct Serve {
    /// The port to listen on
    #[clap(long, default_value("127.0.0.1:38668"))]
    pub addr: Option<std::net::SocketAddr>,
}

impl Serve {
    pub async fn run(&self) {
        debug!("Serve: {self:?}");

        // start server
        let addr = self.addr.unwrap();
        crate::server::start(addr).await.unwrap();
    }
}
