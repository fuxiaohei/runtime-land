use serde::{Deserialize, Serialize};

const VERSION: &str = "1.0";
pub const DEFAULT_FILE: &str = "land.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub name: String,
    pub description: String,
    pub language: String,
    pub version: String,
    pub build: BuildData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildData {
    pub main: String,
    pub cmd: String,
}

impl Data {
    pub fn new_js() -> Data {
        Data {
            name: "js".to_string(),
            description: "Javascript".to_string(),
            language: "js".to_string(),
            version: VERSION.to_string(),
            build: BuildData {
                main: "src/index.js".to_string(),
                cmd: "".to_string(),
            },
        }
    }
}
