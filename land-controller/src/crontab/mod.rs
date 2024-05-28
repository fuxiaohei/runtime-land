use tracing::error;

mod deploy;
mod traffic;

pub fn init() {
    // init traffic refresh to database
    tokio::spawn(async {
        // every 20min
        let interval = tokio::time::Duration::from_secs(20 * 60);
        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await;
        loop {
            match traffic::refresh().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Traffic refresh failed: {}", e);
                }
            };
            ticker.tick().await;
        }
    });

    // handle deploy tasks in every second
    tokio::spawn(async {
        let interval = tokio::time::Duration::from_secs(1);
        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await;
        loop {
            deploy::run_tasks().await.unwrap_or_else(|e| {
                error!("Deploy tasks failed: {}", e);
            });
            ticker.tick().await;
        }
    });
}
