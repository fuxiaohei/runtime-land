use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpConfig {
    pub addr: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:38779".to_string(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub http: HttpConfig,
}

impl Config {
    /// read config from toml file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}