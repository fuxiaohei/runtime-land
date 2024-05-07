use tracing::warn;

mod deploying;
mod traffic;
mod waiting;
pub use traffic::{
    query_flows_traffic, query_requests_traffic, refresh_projects, TrafficPeriodParams,
};

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
            // every 20 minute to refresh project traffic data
            tokio::time::sleep(tokio::time::Duration::from_secs(1200)).await;
            match traffic::refresh().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Metrics::refresh failed: {}", e);
                }
            }
        }
    });
}
