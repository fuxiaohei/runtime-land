mod gen_deploys;
pub use gen_deploys::get as get_deploys;
pub use gen_deploys::{ConfData, ConfItem};
use tracing::warn;

mod livings_worker;
mod review_tasks;

pub struct Options {
    pub gen_deploys: u64,
    pub review_tasks: u64,
    pub livings_worker: u64,
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

fn start_review_tasks(interval: u64) {
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

fn start_check_livings_worker(interval: u64) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval));
        loop {
            ticker.tick().await;
            match livings_worker::cron().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Check livings worker error: {:?}", e);
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
    if opt.livings_worker > 0 {
        start_check_livings_worker(opt.livings_worker);
    }
}
