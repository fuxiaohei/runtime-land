use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::error;

mod generate;
pub mod traefik;

/// DATA is a global variable to store deployment data
pub static DATA: Lazy<Mutex<ConfData>> = Lazy::new(|| {
    Mutex::new(ConfData {
        items: vec![],
        checksum: "".to_string(),
    })
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfItem {
    pub user_id: i32,
    pub project_id: i32,
    pub path: String,
    pub dl_url: String,
    pub status: String,
    pub md5: String,
    pub domain: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfData {
    pub items: Vec<ConfItem>,
    pub checksum: String,
}

/// generate_loop will generate a new token every `seconds` seconds
pub fn generate_loop(seconds: i32) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(seconds as u64));
        loop {
            ticker.tick().await;
            match generate::generate().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Generate loop error: {:?}", e);
                }
            }
        }
    });
}
