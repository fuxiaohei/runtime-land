use envconfig::Envconfig;
use land_core::confdata::RuntimeData;
use once_cell::sync::OnceCell;
use sysinfo::{CpuExt, System, SystemExt};
use tracing::{debug, info, instrument, warn};

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "EDGE_ADDR", default = "http://127.0.0.1:7902")]
    addr: String,
    #[envconfig(from = "EDGE_SYNC_ENABLED", default = "true")]
    sync_enabled: bool,
    #[envconfig(from = "EDGE_SYNC_INTERVAL", default = "5")]
    sync_interval: u64,
}

/// SERVER_NAME is the name of region
pub static SERVER_NAME: OnceCell<String> = OnceCell::new();

#[instrument(skip_all, name = "[EDGE]")]
async fn sync_interval(cfg: Config) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(cfg.sync_interval));

    let mut sys = System::new_all();
    let url = format!("{}/v1/sync", cfg.addr);
    let client = reqwest::Client::new();

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

        let resp = match client.post(&url).json(&data).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("sync request failed: {}", e);
                continue;
            }
        };
        if !resp.status().is_success() {
            warn!("sync responst status failed: {}", resp.status());
            // sleep 5s to retry
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }

        let recv = match resp.json::<land_core::confdata::RuntimeRecvData>().await {
            Ok(recv) => recv,
            Err(e) => {
                warn!("sync response json failed: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        debug!("sync recv: {:?}", recv);
        SERVER_NAME.get_or_init(|| recv.region_name);
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
