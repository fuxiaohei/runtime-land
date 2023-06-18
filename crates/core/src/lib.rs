use anyhow::Result;
use once_cell::sync::OnceCell;
use tracing::info;

pub mod dao;
pub mod db;
pub mod meta;
pub mod model;
pub mod region;
pub mod storage;
pub mod trace;
pub mod version;

pub static PROD_DOMAIN: OnceCell<String> = OnceCell::new();
pub static PROD_PROTOCOL: OnceCell<String> = OnceCell::new();

// init_prod_const initializes the PROD_DOMAIN const
#[tracing::instrument(name = "[PROD_DOMAIN]")]
pub async fn init_prod_const() -> Result<()> {
    let domain = std::env::var("PROD_DOMAIN").unwrap_or("127-0-0-1.nip.io".to_string());
    let protocol = std::env::var("PROD_PROTOCOL").unwrap_or("http".to_string());
    info!("Set {}://{}", protocol, domain);
    PROD_DOMAIN.set(domain).unwrap();
    PROD_PROTOCOL.set(protocol).unwrap();
    Ok(())
}
