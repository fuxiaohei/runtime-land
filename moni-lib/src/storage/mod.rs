use anyhow::{anyhow, Result};
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use opendal::Operator;

use self::local::init_local;

mod local;

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "MONI_STORAGE_TYPE", default = "local")]
    pub type_name: String,
}

pub static STORAGE: OnceCell<Operator> = OnceCell::new();
pub static STORAGE_PREFIX: OnceCell<String> = OnceCell::new();

pub async fn init() -> Result<()> {
    let cfg = Config::init_from_env().unwrap();
    match cfg.type_name.as_str() {
        "local" => {
            let op = init_local().await?;
            STORAGE.set(op).unwrap();
            STORAGE_PREFIX.set("local://".to_string()).unwrap();
        }
        _ => {
            return Err(anyhow!("unknown storage type: {}", cfg.type_name));
        }
    }

    Ok(())
}

// get_prefix returns the storage prefix
pub fn get_prefix() -> String {
    STORAGE_PREFIX.get().unwrap().clone()
}
