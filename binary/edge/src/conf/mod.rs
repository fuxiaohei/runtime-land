use anyhow::Result;
use land_core::confdata::{RouteConfItem, RoutesConf};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

mod operator;
mod store;
mod traefik;

lazy_static! {
    pub static ref CONF_VALUES: Mutex<RoutesConf> = Mutex::new(RoutesConf {
        items: vec![],
        created_at: 0,
    });
}

/// CONF_LOCAL_FILE is the local file name for conf
const CONF_LOCAL_FILE: &str = "edge-conf.json";

/// init conf
#[instrument(name = "[Conf]")]
pub async fn init() -> Result<()> {
    // init store
    store::init().await?;

    // check local conf file exist
    if exist_conf().await? {
        // read local conf file to global conf
        let conf = read_conf().await?;
        let mut local_conf = CONF_VALUES.lock().await;
        local_conf.items = conf.items;
        local_conf.created_at = conf.created_at;
        info!("init local conf version: {}", local_conf.created_at);
    } else {
        info!("conf file not exist");
    }

    // init operator
    operator::init_operator().await?;

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
    debug!("remote conf: {:?}", remote_conf.items.len());

    let mut local_conf = CONF_VALUES.lock().await;
    if local_conf.created_at > remote_conf.created_at {
        warn!("local conf is newer than remote conf, ignore");
        return;
    }

    // compare remote conf and local conf
    let (updates, removes) = compare_conf(&remote_conf, &local_conf).await;

    info!("updates: {:?}", updates.len());
    info!("removes: {:?}", removes.len());

    // deploy updates
    let operator = operator::OPERATOR.get().unwrap();
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
    // delete removes
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

    // update local conf
    local_conf.items = remote_conf.items;
    local_conf.created_at = remote_conf.created_at;
    match write_conf(&local_conf).await {
        Ok(_) => {}
        Err(e) => {
            warn!("write conf error: {:?}", e);
        }
    }
}

async fn write_conf(conf: &RoutesConf) -> Result<()> {
    let s = store::LOCAL_STORE.get().unwrap();
    let data = serde_json::to_vec(conf)?;
    s.write(CONF_LOCAL_FILE, data).await?;
    Ok(())
}

async fn read_conf() -> Result<RoutesConf> {
    let s = store::LOCAL_STORE.get().unwrap();
    let data = s.read(CONF_LOCAL_FILE).await?;
    let conf: RoutesConf = serde_json::from_slice(&data)?;
    Ok(conf)
}

async fn exist_conf() -> Result<bool> {
    let s = store::LOCAL_STORE.get().unwrap();
    let exist = s.is_exist(CONF_LOCAL_FILE).await?;
    Ok(exist)
}
