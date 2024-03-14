mod gen_deploys;
pub use gen_deploys::get as get_deploys;
pub use gen_deploys::{ConfData, ConfItem};

pub struct Options {
    pub gen_deploys: u64,
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

pub fn start(opt: Options) {
    if opt.gen_deploys > 0 {
        start_gen_deploys(opt.gen_deploys);
    }
}
