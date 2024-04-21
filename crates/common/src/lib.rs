use serde::{Deserialize, Serialize};

pub mod tracing;
pub mod version;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IPInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub hostname: Option<String>,
}
