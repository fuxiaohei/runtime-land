use envconfig::Envconfig;
use land_core::confdata::RuntimeData;
use sysinfo::{CpuExt, System, SystemExt};
use tracing::{debug, info, instrument, warn};

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "EDGE_ADDR", default = "http://127.0.0.1:7899")]
    addr: String,
    #[envconfig(from = "EDGE_SYNC_ENABLED", default = "true")]
    sync_enabled: bool,
    #[envconfig(from = "EDGE_SYNC_INTERVAL", default = "1")]
    sync_interval: u64,
}

#[instrument(skip_all, name = "[EDGE]")]
async fn sync_interval(cfg: Config) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(cfg.sync_interval));

    let mut sys = System::new_all();
    let url = format!("{}/v1/sync", cfg.addr);

    loop {
        interval.tick().await;

        let hostname = gethostname::gethostname();
        sys.refresh_cpu();
        let cpu_count = sys.physical_core_count().ok_or(0).unwrap();
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        sys.refresh_memory();
        let total_memory = sys.total_memory() / 1024 / 1024;
        let used_memory = sys.used_memory() / 1024 / 1024;

        let now_ts = chrono::Utc::now().timestamp() as u64;
        let data = RuntimeData {
            hostname: hostname.into_string().unwrap(),
            cpu_count,
            cpu_usage,
            total_memory,
            used_memory,
            updated_at: now_ts,
        };
        let client = reqwest::Client::new();
        let resp = match client.post(&url).json(&data).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("sync request failed: {}", e);
                continue;
            }
        };
        if resp.status().is_success() {
            debug!("sync success");
        } else {
            warn!("sync responst status failed: {}", resp.status());
        }
    }
}

#[instrument(name = "[EDGE]")]
pub async fn init() {
    let cfg = Config::init_from_env().unwrap();
    info!("Load args: {:?}", cfg);
    if !cfg.sync_enabled {
        warn!("sync disabled");
    }
    tokio::spawn(sync_interval(cfg));
}
