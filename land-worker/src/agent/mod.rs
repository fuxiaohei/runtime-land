use anyhow::Result;

pub mod ip;
pub mod sync;
pub mod traefik;

/// run is agent runner
pub async fn run(addr: String, token: String, dir: String) -> Result<()> {
    sync::start(1, addr, token, dir).await;
    Ok(())
}
