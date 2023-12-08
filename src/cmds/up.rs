use clap::Args;
use land_common::{MetaData, MANIFEST_FILE};
use land_worker_server::Opts;
use std::net::SocketAddr;

#[derive(Args, Debug)]
pub struct Up {
    #[clap(long = "listen", value_parser = validate_address,default_value("127.0.0.1:3030"))]
    pub address: Option<String>,
}

impl Up {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let metadata = MetaData::from_file(MANIFEST_FILE)?;
        let target = std::path::Path::new(&metadata.build.target);
        if !target.exists() {
            return Err(anyhow::anyhow!(
                "Fail to load Wasm target '{}'!",
                &metadata.build.target,
            ));
        }

        // set options
        let current_dir = std::env::current_dir()?;
        let server_opts = Opts {
            addr: self.address.clone().unwrap().parse().unwrap(),
            dir: current_dir.to_str().unwrap().to_string(),
            default_wasm: metadata.build.target,
            endpoint_name: "land-cli".to_string(),
        };
        land_worker_server::run(server_opts).await?;
        Ok(())
    }
}

fn validate_address(listen: &str) -> Result<String, String> {
    let _: SocketAddr = listen
        .parse()
        .map_err(|_| "invalid listen address".to_string())?;
    Ok(listen.to_string())
}
