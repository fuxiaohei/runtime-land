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
}

#[cfg(test)]
mod tests{
    use super::MetaData;

    #[test]
    fn test_metadata_toml(){
        let metadata = MetaData::new_for_rust();
        let toml = toml::to_string(&metadata).unwrap();
        println!("{}", toml);
    }
}