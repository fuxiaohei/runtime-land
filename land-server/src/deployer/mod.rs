use tracing::warn;

mod deploying;
mod metrics;
mod waiting;

/// run_background starts the background worker to handle the deployer's tasks.
pub fn run_background() {
    tokio::spawn(async {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            match waiting::run_tasks().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Waiting::run_tasks failed: {}", e);
                }
            };
            match deploying::run_tasks().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Deploying::run_tasks failed: {}", e);
                }
            }
        }
    });
    tokio::spawn(async {
        // every 20 minute to refresh project traffic data
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1200));
        loop {
            ticker.tick().await;
            match metrics::refresh().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Metrics::refresh failed: {}", e);
                }
            }
        }
    });
}
