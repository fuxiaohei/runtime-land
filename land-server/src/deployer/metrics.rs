use anyhow::Result;
use tracing::info;

/// refresh refreshes the metrics
pub async fn refresh() -> Result<()> {
    info!("Metrics::refresh");
    Ok(())
}
