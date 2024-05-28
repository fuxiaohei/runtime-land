use tracing::warn;

mod envs;

/// run_background starts the background worker to handle the deployer's tasks.
pub fn run_background() {
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
