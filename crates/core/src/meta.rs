use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// DEFAULT_METADATA_FILE is the default Meta file name
pub const DEFAULT_METADATA_FILE: &str = "meta.toml";

/// Meta is the Meta struct
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meta {
    pub manifest: String,
    pub project: String, // project name auto generated
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<MetadataBuild>,
}

/// MetadataBuild is the build section of the Meta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataBuild {
    pub rust_target_dir: Option<String>,
}

impl Meta {
    /// read Meta from toml file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut manifest: Meta = toml::from_str(&content)?;

        // fill value to default for Option<T>
        if manifest.build.is_none() {
            manifest.build = Some(MetadataBuild::default());
        }

        Ok(manifest)
    }

    /// read Meta from binary
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let manifest: Meta = toml::from_str(std::str::from_utf8(data)?)?;
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

    /// is wasi
    pub fn is_wasi(&self) -> bool {
        self.get_arch() == "wasm32-wasi"
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

    /// get output file
    pub fn get_output(&self) -> String {
        self.get_target().replace(".wasm", ".component.wasm")
    }

    /// get src directory name
    pub fn get_src_dir(&self) -> String {
        if self.language == "js" {
            return "dist/".to_string();
        }
        "src/".to_string()
    }

    /// get project name
    pub fn get_project_name(&self) -> String {
        self.project.clone()
    }

    /// generate project name with random string
    pub fn generate_project_name(&mut self) -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        self.project = format!("{}-{}", self.name, rand_string.to_lowercase());
        self.project.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// test manifest from_file
    #[test]
    fn from_file() {
        let manifest = Meta::from_file("../../tests/data/meta.toml").unwrap();
        assert_eq!(manifest.manifest, "v1");
        assert_eq!(manifest.name, "rust-basic");
        assert_eq!(manifest.description, "example rust project");
        assert_eq!(manifest.authors, vec!["your-name"]);
        assert_eq!(manifest.language, "rust");
        assert_eq!(
            manifest.build.as_ref().unwrap().rust_target_dir,
            Some("./target".to_string())
        );
    }

    /// test manifest to file
    #[test]
    fn to_file() {
        let manifest = Meta::from_file("../../tests/data/meta.toml").unwrap();
        manifest.to_file("../../tests/data/meta2.toml").unwrap();
        let manifest2 = Meta::from_file("../../tests/data/meta2.toml").unwrap();
        assert_eq!(manifest.manifest, manifest2.manifest);
        assert_eq!(manifest.name, manifest2.name);
        assert_eq!(manifest.description, manifest2.description);
        assert_eq!(manifest.authors, manifest2.authors);
        assert_eq!(manifest.language, manifest2.language);
        assert_eq!(
            manifest.build.as_ref().unwrap().rust_target_dir,
            manifest2.build.as_ref().unwrap().rust_target_dir
        );
        std::fs::remove_file("../../tests/data/meta2.toml").unwrap();
    }
}
