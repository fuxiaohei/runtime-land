use ::tracing::info;
use anyhow::Result;
use once_cell::sync::OnceCell;

pub mod dao;
pub mod db;
pub mod meta;
pub mod model;
pub mod storage;
pub mod tracing;
pub mod version;

pub static PROD_DOMAIN: OnceCell<String> = OnceCell::new();

// init_prod_const initializes the PROD_DOMAIN const
pub async fn init_prod_const() -> Result<()> {
    let domain = std::env::var("MONI_PROD_DOMAIN").unwrap_or("127-0-0-1.nip.io".to_string());
    info!("set PROD_DOMAIN to {}", domain);
    PROD_DOMAIN.set(domain).unwrap();
    Ok(())
}
