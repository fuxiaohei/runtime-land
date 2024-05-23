use anyhow::Result;
use clap::Args;
use tracing::info;

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    #[clap(short = 'j', long = "js-engine")]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) -> Result<()> {
        info!("Build command: {:?}", self);
        Ok(())
    }
}
