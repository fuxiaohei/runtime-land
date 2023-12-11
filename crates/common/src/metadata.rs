use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const MANIFEST_VERSION: &str = "0.2.0";
pub const MANIFEST_FILE: &str = "land.toml";

#[derive(Serialize, Deserialize)]
pub struct MetaData {
    #[serde(rename = "land_manifest_version")]
    pub manifest_version: String,
    pub project: ProjectMetaData,
    pub build: BuildMetaData,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMetaData {
    pub name: String,
    pub version: String,
    pub language: String,
    pub authors: Vec<String>,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct BuildMetaData {
    pub command: String,
    pub target: String,
}

impl MetaData {
    pub fn new_for_rust() -> Self {
        MetaData {
            manifest_version: MANIFEST_VERSION.to_string(),
            project: ProjectMetaData {
                name: "".to_string(),
                version: "0.1.0".to_string(),
                language: "rust".to_string(),
                authors: vec![],
                description: "".to_string(),
            },
            build: BuildMetaData {
                command: "cargo build --release".to_string(),
                target: "target/wasm32-wasi/release/".to_string(),
            },
        }
    }

    pub fn new_for_js() -> Self {
        MetaData {
            manifest_version: MANIFEST_VERSION.to_string(),
            project: ProjectMetaData {
                name: "".to_string(),
                version: "0.1.0".to_string(),
                language: "javascript".to_string(),
                authors: vec![],
                description: "".to_string(),
            },
            build: BuildMetaData {
                command: "".to_string(),
                target: "dist/".to_string(),
            },
        }
    }

    /// Parse metadata from string
    pub fn from_string(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).map_err(|e| anyhow::anyhow!(e))
    }

    /// Read metadata from file
    pub fn from_file(path: &str) -> Result<Self> {
        if !std::path::Path::new(path).exists() {
            return Err(anyhow::anyhow!("Metadata file '{}' not found!", path));
        }
        let toml_str = std::fs::read_to_string(path)?;
        let data = Self::from_string(&toml_str)?;
        data.validate()?;
        Ok(data)
    }

    /// validate metadata
    pub fn validate(&self) -> Result<()> {
        // TODO: check metadata fields are valid
        Ok(())
    }
}

/// js_real_target_path returns the real target path
pub fn js_real_target_path(path: &str) -> String {
    path.replace(".js", ".wasm").to_string()
}

#[cfg(test)]
mod tests {
    use super::MetaData;

    #[test]
    fn test_metadata_toml() {
        let metadata = MetaData::new_for_rust();
        let toml = toml::to_string(&metadata).unwrap();
        println!("{}", toml);
    }
}
