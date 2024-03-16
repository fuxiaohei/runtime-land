use anyhow::Result;
use tracing::{info, instrument};

#[instrument["[LivW]"]]
pub async fn cron() -> Result<()> {
    let livings = land_dao::worker::list_online().await?;
    let now_ts = chrono::Utc::now().timestamp();
    for l in livings {
        let ut = l.updated_at.and_utc().timestamp();
        // 1 minute
        if now_ts - ut > 60 {
            info!(id = l.id, ip = l.ip, "Set offline");
            land_dao::worker::set_offline(l.id).await?;
        }
    }
    Ok(())
}
