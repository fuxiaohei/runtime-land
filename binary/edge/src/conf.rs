use land_core::confdata::RoutesConf;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref CURRENT_CONF_VERSION: Mutex<u64> = Mutex::new(0);
}

pub async fn process_conf(conf: &RoutesConf) {
    println!("process conf: {:?}", conf);

    let mut version = CURRENT_CONF_VERSION.lock().await;
    *version = conf.created_at;
}
