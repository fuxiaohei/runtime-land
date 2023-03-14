use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CliEnv {
    pub api_key: String,
    pub api_jwt_token: String,
    pub api_host: String,
}

impl CliEnv {
    pub fn to_file(&self, path: &str) -> Result<()> {
        // use bincode
        let content = bincode::serialize(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read(path)?;
        let env: CliEnv = bincode::deserialize(&content)?;
        Ok(env)
    }
}

/// DEFAULT_ENV_FILE is the default env file name
pub const DEFAULT_ENV_FILE: &str = ".moni-cli.env";

/// get Meta env file from home path
pub fn get_metadata_env_file() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home)
        .join(DEFAULT_ENV_FILE)
        .to_str()
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    /// test Meta env
    #[test]
    fn env() {
        let env_data = CliEnv {
            api_key: "api_key".to_string(),
            api_host: "api_host".to_string(),
            api_jwt_token: "api_jwt_token".to_string(),
        };
        env_data.to_file("../tests/data/cli.env").unwrap();
        let env_data2 = CliEnv::from_file("../tests/data/cli.env").unwrap();
        assert_eq!(env_data.api_key, env_data2.api_key);
        assert_eq!(env_data.api_jwt_token, env_data2.api_jwt_token);
        assert_eq!(env_data.api_host, env_data2.api_host);

        std::fs::remove_file("../tests/data/cli.env").unwrap();
    }
}
