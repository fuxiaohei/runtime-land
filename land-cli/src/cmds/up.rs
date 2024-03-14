use anyhow::Result;
use clap::Args;
use land_common::manifest::{Data, MANIFEST_FILE};
use land_worker_server::Opts;
use std::net::SocketAddr;

use super::build::build_internal;

#[derive(Args, Debug)]
pub struct Up {
    #[clap(long = "listen", value_parser = validate_address,default_value("127.0.0.1:3030"))]
    pub address: Option<String>,
    #[clap(long = "build")]
    pub build: bool,
}

fn validate_address(listen: &str) -> Result<String, String> {
    let _: SocketAddr = listen
        .parse()
        .map_err(|_| "invalid listen address".to_string())?;
    Ok(listen.to_string())
}

impl Up {
    pub async fn run(&self) -> Result<()> {
        let metadata = Data::from_file(MANIFEST_FILE)?;
        if self.build {
            build_internal(&metadata)?;
        }
        if !std::path::Path::new(&metadata.build.target).exists() {
            return Err(anyhow::anyhow!(
                "Build target '{}' does not exist!",
                &metadata.build.target,
            ));
        }
        let current_dir = std::env::current_dir()?;
        let wasm_target = metadata.wasm_target();

        let opts = Opts {
            addr: self.address.clone().unwrap().parse::<SocketAddr>().unwrap(),
            dir: current_dir.to_str().unwrap().to_string(),
            default_wasm: wasm_target,
            endpoint_name: Some("localhost".to_string()),
            wasm_aot: false,
        };
        land_worker_server::start(opts)
            .await
            .expect("run worker server");
        Ok(())
    }
}
