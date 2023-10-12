use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// DEFAULT_FILE is the default Meta file name
pub const DEFAULT_FILE: &str = "land.toml";

/// Meta is the Meta struct
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub manifest: String,
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<MetadataBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_name: Option<String>,
}

/// MetadataBuild is the build section of the Meta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataBuild {
    pub rust_target_dir: Option<String>,
    pub rust_src_dir: Option<Vec<String>>,
}

impl Metadata {
    /// read Meta from toml file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut manifest: Metadata = toml::from_str(&content)?;

        // fill value to default for Option<T>
        if manifest.build.is_none() {
            manifest.build = Some(MetadataBuild::default());
        }

        Ok(manifest)
    }

    /// read Meta from binary
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let manifest: Metadata = toml::from_str(std::str::from_utf8(data)?)?;
        Ok(manifest)
    }

    /// write Meta to toml file
    pub fn to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// get arch from Meta
    pub fn get_arch(&self) -> String {
        "wasm32-wasi".to_string()
    }

    /// get compiled target
    pub fn get_target(&self) -> String {
        let target = self
            .build
            .clone()
            .unwrap_or_default()
            .rust_target_dir
            .unwrap_or_else(|| "target".to_string());
        let arch = self.get_arch();
        let target_dir = Path::new(&target).join(arch).join("release");
        let name = self.name.replace('-', "_") + ".wasm";
        target_dir.join(name).to_str().unwrap().to_string()
    }

    /// get source dir
    pub fn get_source_dirs(&self) -> Vec<String> {
        let src_dir = self
            .build
            .clone()
            .unwrap_or_default()
            .rust_src_dir
            .unwrap_or_else(|| vec!["src".to_string()]);
        src_dir
    }

    /// get output file
    pub fn get_output(&self) -> String {
        self.get_target().replace(".wasm", ".component.wasm")
    }
}
