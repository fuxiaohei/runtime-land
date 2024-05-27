use tracing::warn;

mod deploying;
mod envs;
mod waiting;

/// run_background starts the background worker to handle the deployer's tasks.
pub fn run_background() {
    tokio::spawn(async {
        loop {
            // sleep 1 second
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
            };
        }
    });
    tokio::spawn(async {
        loop {
            match envs::refresh().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Envs::refresh failed: {}", e);
                }
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });
}
