use anyhow::Result;
use async_trait::async_trait;
use land_core::confdata::{RouteConfItem, RoutesConf};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::{info, instrument, warn};

mod traefik;

lazy_static! {
    pub static ref CONF_VALUES: Mutex<RoutesConf> = Mutex::new(RoutesConf {
        items: vec![],
        created_at: 0,
    });
}

/// CONF_LOCAL_FILE is the local file name for conf
const CONF_LOCAL_FILE: &str = "edge-conf.json";

/// OPERATOR is the conf operator
pub static OPERATOR: once_cell::sync::OnceCell<Box<dyn ConfOperatorTrait + Send + Sync>> =
    once_cell::sync::OnceCell::new();

/// init conf
#[instrument(name = "[Conf]")]
pub async fn init() -> Result<()> {
    // check local conf file exist
    if std::path::Path::new(CONF_LOCAL_FILE).exists() {
        // read local conf file to global conf
        let conf = read_conf()?;
        let mut local_conf = CONF_VALUES.lock().await;
        local_conf.items = conf.items;
        local_conf.created_at = conf.created_at;
        info!("init conf version: {}", local_conf.created_at);
    } else {
        info!("conf file not exist");
    }

    // init operator
    let operator_type =
        std::env::var("CONF_OPERATOR_TYPE").unwrap_or_else(|_| "traefik-redis".to_string());
    info!("operator type: {}", operator_type);
    match operator_type.as_str() {
        "traefik-redis" => {
            let mut op = traefik::TraefikOperator::new();
            op.init().await?;
            OPERATOR
                .set(Box::new(op))
                .map_err(|_| anyhow::anyhow!("set operator error"))?;
            info!("init operator: {:?}", operator_type);
        }
        _ => {
            return Err(anyhow::anyhow!("operator unknown"));
        }
    }
    Ok(())
}

async fn compare_conf(
    remote_conf: &RoutesConf,
    local_conf: &RoutesConf,
) -> (Vec<RouteConfItem>, Vec<RouteConfItem>) {
    let remote_map = remote_conf.to_map();
    let local_map = local_conf.to_map();

    let mut updates = vec![];

    // if remote map item in local map, check md5 equal. if not equal, need update
    // if remote map item not in local map, need update
    for (k, v) in &remote_map {
        if let Some(local_v) = local_map.get(k) {
            if v.md5 != local_v.md5 {
                updates.push(v.clone());
            }
        } else {
            updates.push(v.clone());
        }
    }

    let mut removes = vec![];
    // if local map item not in remote map, need remove
    for (k, v) in local_map {
        if !remote_map.contains_key(&k) {
            removes.push(v);
        }
    }

    (updates, removes)
}

pub async fn process_conf(remote_conf: RoutesConf) {
    println!("process conf: {:?}", remote_conf);

    let mut local_conf = CONF_VALUES.lock().await;
    if local_conf.created_at > remote_conf.created_at {
        warn!("local conf is newer than remote conf, ignore");
        return;
    }

    let (updates, removes) = compare_conf(&remote_conf, &local_conf).await;

    info!("updates: {:?}", updates);
    info!("removes: {:?}", removes);

    let operator = OPERATOR.get().unwrap();

    for item in updates {
        match operator.deploy(item.clone()).await {
            Ok(_) => {
                info!("deploy success, domain: {}", item.domain);
            }
            Err(e) => {
                warn!("deploy error: {:?}, domain: {}", e, item.domain);
            }
        }
    }

    for item in removes {
        match operator.remove(item.clone()).await {
            Ok(_) => {
                info!("remove success, domain: {}", item.domain);
            }
            Err(e) => {
                warn!("remove error: {:?}, domain: {}", e, item.domain);
            }
        }
    }

    local_conf.items = remote_conf.items;
    local_conf.created_at = remote_conf.created_at;
    match write_conf(&local_conf) {
        Ok(_) => {}
        Err(e) => {
            warn!("write conf error: {:?}", e);
        }
    }
}

fn write_conf(conf: &RoutesConf) -> Result<()> {
    let json = serde_json::to_string(conf)?;
    std::fs::write(CONF_LOCAL_FILE, json)?;
    Ok(())
}

fn read_conf() -> Result<RoutesConf> {
    let json = std::fs::read_to_string(CONF_LOCAL_FILE)?;
    let conf: RoutesConf = serde_json::from_str(&json)?;
    Ok(conf)
}

#[async_trait]
pub trait ConfOperatorTrait {
    async fn init(&mut self) -> Result<()>;
    async fn deploy(&self, item: RouteConfItem) -> Result<()>;
    async fn remove(&self, item: RouteConfItem) -> Result<()>;
}
