use land_core_service::metrics::traffic::refresh;
use tracing::error;

pub fn init() {
    // init traffic refresh to database
    tokio::spawn(async {
        // every 20min
        let interval = tokio::time::Duration::from_secs(20 * 60);
        let mut ticker = tokio::time::interval(interval);
        loop {
            match refresh().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Traffic refresh failed: {}", e);
                }
            };
            ticker.tick().await;
        }
    });
}
