use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const MANIFEST_VERSION: &str = "0.4.0";
pub const MANIFEST_FILE: &str = "land.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub language: String,
    pub authors: Vec<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub command: String,
    pub target: String,
    pub src_files: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "land_manifest_version")]
    pub manifest_version: String,
    pub project: Project,
    pub build: Build,
}

impl Data {
    pub fn rust() -> Self {
        Data {
            manifest_version: MANIFEST_VERSION.to_string(),
            project: Project {
                name: "".to_string(),
                version: "0.1.0".to_string(),
                language: "rust".to_string(),
                authors: vec![],
                description: "".to_string(),
            },
            build: Build {
                command: "cargo build --release".to_string(),
                target: "target/release/land".to_string(),
                src_files: None,
            },
        }
    }

    pub fn js() -> Self {
        Data {
            manifest_version: MANIFEST_VERSION.to_string(),
            project: Project {
                name: "".to_string(),
                version: "0.1.0".to_string(),
                language: "js".to_string(),
                authors: vec![],
                description: "".to_string(),
            },
            build: Build {
                command: "".to_string(),
                target: "dist/".to_string(),
                src_files: None,
                /*src_files: Some(
                    ["src/", "package.json", "package-lock.json"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),*/
            },
        }
    }

    /// from_string creates a Data from a toml string
    pub fn from_string(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).map_err(|e| anyhow!(e))
    }

    /// from_file creates a Data from a toml file
    pub fn from_file(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            return Err(anyhow!("Metadata file '{}' not found!", path));
        }
        let toml_str = fs::read_to_string(path)?;
        let data = Self::from_string(&toml_str)?;
        // data.validate()?;
        Ok(data)
    }

    /// wasm_target return wasm target file name
    pub fn wasm_target(&self) -> String {
        match self.project.language.as_str() {
            "javascript" | "js" => self.build.target.replace(".js", ".js.wasm"),
            _ => self.build.target.clone(),
        }
    }
}

/// Local is used to store user data in local
#[derive(Debug, Serialize, Deserialize)]
pub struct Local {
    pub id: i32,
    pub token: String,
    pub name: String,
    pub uuid: String,
    pub email: String,
    pub expires: i64,
}
