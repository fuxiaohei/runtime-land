use clap::Args;
use tracing::debug;
use anyhow::Result;

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    #[clap(short = 'j', long = "js-engine")]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) -> Result<()> {
        debug!("Build command: {:?}", self);

        Ok(())
    }
}
