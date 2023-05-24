use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing::{debug, debug_span, warn, Instrument};

pub mod rt_server;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpConfig {
    pub addr: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:38889".to_string(),
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

#[derive(Parser, Debug)]
#[clap(name = "moni-runtime", version = moni_lib::version::get())]
struct Cli {
    /// The conf file
    #[clap(long, default_value("moni-runtime.toml"))]
    pub conf: Option<String>,
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();
    let conf_file = args.conf.as_ref().unwrap();
    // if conf_file is not exist, warn and exit
    if !std::path::Path::new(conf_file).exists() {
        warn!("conf file {} is not exist", conf_file);
        std::process::exit(1);
    }

    let conf = Config::from_file(conf_file).unwrap();
    debug!("load conf: {:?}", conf);

    rt_server::start(conf.http.addr.parse().unwrap())
        .instrument(debug_span!("[Http]"))
        .await
        .unwrap();
}
