use anyhow::Result;
use clap::Args;
use color_print::cprintln;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct Login {
    pub token: String,
    #[clap(long = "url", value_parser = validate_url,default_value("https://cloud.runtime.land"))]
    pub cloud_server_url: Option<String>,
}

/// LoginResponse is the response for /cli/login
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_token: String,
    pub user_name: String,
    pub user_uuid: String,
    pub user_email: String,
}

impl Login {
    pub async fn run(&self) -> Result<()> {
        let login_url = format!(
            "{}/api/v2/cli/login/{}",
            self.cloud_server_url.as_ref().unwrap(),
            self.token
        );
        let resp: LoginResponse = ureq::post(&login_url).call()?.into_json()?;
        // write this resp to ~/.runtimeland/config
        let config_path = get_local_config_path()?;
        let config_str = serde_json::to_string(&resp)?;
        std::fs::write(config_path, config_str)?;
        cprintln!(
            "<bright-cyan,bold>Login Success</> as '{}'({}).",
            resp.user_name,
            resp.user_email
        );
        Ok(())
    }
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}

/// get_local_config_path returns the path of local config file
pub fn get_local_config_path() -> Result<String> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home dir not found"))?;
    let config_dir = home_dir.join(".runtimeland");
    std::fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("config");
    Ok(config_file.to_str().unwrap().to_string())
}

/// get_local_config returns the local config
pub fn get_local_config() -> Option<LoginResponse> {
    let config_path = get_local_config_path().ok()?;
    let config_str = std::fs::read_to_string(config_path).ok()?;
    let config: LoginResponse = serde_json::from_str(&config_str).ok()?;
    Some(config)
}
