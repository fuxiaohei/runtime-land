use anyhow::Result;
use clap::Args;
use land_core::meta;
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

        let meta = meta::Data::from_file(meta::DEFAULT_FILE)?;
        debug!("Meta: {:?}", meta);

        // build again
        if self.build {
            super::build::build_internal(&meta, self.js_engine.clone())?;
        }

        let dist_wasm_path = meta.target_wasm_path();
        // Start server
        let opts = land_wasm_server::Opts {
            addr: self.address.clone().unwrap().parse()?,
            dir: "./".to_string(),
            default_wasm: Some(dist_wasm_path),
            enable_wasmtime_aot: false,
            endpoint_name: Some("localhost".to_string()),
            enable_metrics: false,
            metrics_addr: None,
        };
        land_wasm_server::start(opts).await?;
        Ok(())
    }
}
