mod gen_deploys;
pub use gen_deploys::get as get_deploys;
pub use gen_deploys::{ConfData, ConfItem};
use tracing::warn;

mod review_tasks;

pub struct Options {
    pub gen_deploys: u64,
    pub review_tasks: u64,
}

fn start_gen_deploys(seconds: u64) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(seconds));
        loop {
            ticker.tick().await;
            gen_deploys::cron().await;
        }
    });
}

pub fn start_review_tasks(interval: u64) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval));
        loop {
            ticker.tick().await;
            match review_tasks::cron().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Review tasks error: {:?}", e);
                }
            }
        }
    });
}

pub fn start(opt: Options) {
    if opt.gen_deploys > 0 {
        start_gen_deploys(opt.gen_deploys);
    }
    if opt.review_tasks > 0 {
        start_review_tasks(opt.review_tasks);
    }
}
