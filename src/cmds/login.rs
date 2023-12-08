use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct Login {
    pub token: String,
    #[clap(long = "url", value_parser = validate_url,default_value("https://cloud.runtime.land"))]
    pub cloud_server_url: Option<String>,
}

impl Login {
    pub async fn run(&self) -> Result<()> {
        println!("login: {:?}", self);
        Ok(())
    }
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}
