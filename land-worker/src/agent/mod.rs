use anyhow::Result;

pub mod ip;
pub mod sync;

/// run is agent runner
pub async fn run(addr:String,token:String) -> Result<()> {
    sync::start(1,addr,token).await;
    Ok(())
}
