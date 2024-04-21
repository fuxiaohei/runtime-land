use anyhow::Result;
use tracing::{info, warn};

/// run_background starts the background worker to handle set living workers offline
pub fn run_background() {
    tokio::spawn(async {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            match check_living_workers().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Check living workers failed: {}", e);
                }
            }
        }
    });
}

async fn check_living_workers() -> Result<()> {
    let workers = land_dao::worker::list_online().await?;
    if workers.is_empty() {
        return Ok(());
    }
    for worker in workers {
        let last_alive = worker.updated_at.and_utc();
        let now = chrono::Utc::now();
        let duration = now - last_alive;
        if duration.num_seconds() > 20 {
            land_dao::worker::set_offline(worker.id).await?;
            info!("Set worker offline: {}", worker.ip);
        }
    }
    Ok(())
}
