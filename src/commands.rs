use clap::Args;

/// Command Init
#[derive(Args, Debug)]
pub struct Init {
    /// The name of the project
    pub name: Option<String>,
}

impl Init {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        println!("Init: {:?}", self);
        Ok(())
    }
}
