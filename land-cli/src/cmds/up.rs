use anyhow::Result;
use clap::Args;
use land_core_service::metadata;
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
        let meta = metadata::Data::from_file(metadata::DEFAULT_FILE)?;
        if self.build {
            super::build::build_internal(&meta, self.js_engine.clone())?;
        }
        let dist_wasm_path = meta.target_wasm_path();
        let opts = land_worker_server::Opts {
            addr: self.address.clone().unwrap().parse()?,
            dir: "./".to_string(),
            default_wasm: dist_wasm_path,
            endpoint_name: Some("land-cli".to_string()),
            wasm_aot: false,
            metrics: false,
        };
        land_worker_server::init_globals(&opts)?;
        land_worker_server::start(opts.addr).await?;
        Ok(())
    }
}
