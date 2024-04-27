use tracing::warn;

mod deploying;
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
}
