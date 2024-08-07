use anyhow::Result;
use clap::Args;
use std::net::SocketAddr;
use tracing::debug;

/// Command Up
#[derive(Args, Debug)]
pub struct Up {
    #[clap(long = "listen", value_parser = validate_address,default_value("127.0.0.1:9830"))]
    pub address: Option<String>,
    #[clap(long = "build")]
    pub build: bool,
    #[clap(short = 'j', long = "js-engine")]
    pub js_engine: Option<String>,
}

fn validate_address(listen: &str) -> Result<String, String> {
    let _: SocketAddr = listen
        .parse()
        .map_err(|_| "invalid listen address".to_string())?;
    Ok(listen.to_string())
}

impl Up {
    pub async fn run(&self) -> Result<()> {
        debug!("Up command: {:?}", self);
        Ok(())
    }
}
