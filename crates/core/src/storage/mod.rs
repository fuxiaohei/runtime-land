use anyhow::{anyhow, Result};
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use opendal::Operator;
use tracing::debug;

use self::local::init_local;

mod local;

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "STORAGE_TYPE", default = "local")]
    pub type_name: String,
}

pub static STORAGE: OnceCell<Operator> = OnceCell::new();


#[tracing::instrument(name="[STORAGE]")]
pub async fn init() -> Result<()> {
    let cfg = Config::init_from_env().unwrap();
    debug!("Init storage cfg: {:?}", cfg);
    match cfg.type_name.as_str() {
        "local" => {
            let op = init_local().await?;
            STORAGE.set(op).unwrap();
        }
        _ => {
            return Err(anyhow!("unknown storage type: {}", cfg.type_name));
        }
    }

    Ok(())
}
