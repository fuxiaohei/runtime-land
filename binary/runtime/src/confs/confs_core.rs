use super::endpoint;
use anyhow::Result;
use land_core::confdata::{EndpointConf, RouteConfItem};
use land_core::storage;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

lazy_static! {
    pub static ref CONF_VALUES: Mutex<EndpointConf> = Mutex::new(EndpointConf {
        items: vec![],
        created_at: 0,
        md5: "".to_string(),
    });
}

pub async fn init_conf_file() -> Result<()> {
    // check local conf file exist
    if exist_conf().await? {
        // read local conf file to global conf
        let conf = read_conf().await?;
        let mut local_conf = CONF_VALUES.lock().await;
        local_conf.items = conf.items;
        local_conf.created_at = conf.created_at;
        local_conf.md5 = conf.md5;
        info!(
            "init local conf version: {}, file:{}",
            local_conf.md5, CONF_LOCAL_FILE
        );
    } else {
        info!("conf file not exist");
    }
    Ok(())
}

#[instrument(skip_all, name = "[CONF]")]
pub async fn start_sync(addr: &str, token: &str) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    let client = reqwest::Client::new();
    let endpoint = endpoint::ENDPOINT.get().unwrap().clone();
    loop {
        interval.tick().await;

        let mut conf_values = CONF_VALUES.lock().await;

        let url = format!(
            "{}/v2/endpoint/conf?md5={}&endpoint={}",
            addr, conf_values.md5, endpoint
        );

        let resp = match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("sync conf failed: {}", e);
                continue;
            }
        };
        if resp.status() == 304 {
            debug!("sync conf not modified");
            continue;
        }
        if !resp.status().is_success() {
            warn!(
                "sync conf failed: {}, body:{}",
                resp.status(),
                resp.text().await.unwrap_or("".to_string())
            );
            continue;
        }
        let new_conf_values: EndpointConf = match resp.json().await {
            Ok(value) => value,
            Err(e) => {
                warn!("sync conf json failed: {}", e);
                continue;
            }
        };

        if conf_values.md5 != new_conf_values.md5 {
            info!("update conf: {:?}", new_conf_values.md5);

            process_conf(&conf_values, &new_conf_values).await;

            *conf_values = new_conf_values;
            match write_conf(&conf_values).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("write conf error: {:?}", e);
                }
            }
        }
    }
}

async fn compare_conf(
    remote_conf: &EndpointConf,
    local_conf: &EndpointConf,
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

pub async fn process_conf(local_conf: &EndpointConf, remote_conf: &EndpointConf) {
    debug!("remote conf: {:?}", remote_conf.items.len());

    // compare remote conf and local conf
    let (updates, removes) = compare_conf(remote_conf, local_conf).await;

    info!("updates: {:?}", updates.len());
    info!("removes: {:?}", removes.len());

    // deploy updates
    let operator = super::confs_operator::OPERATOR.get();
    if operator.is_none() {
        warn!("operator not init");
        return;
    }
    let operator = operator.unwrap();
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
}

/// CONF_LOCAL_FILE is the local file name for conf
const CONF_LOCAL_FILE: &str = "endpoint-conf.json";

async fn write_conf(conf: &EndpointConf) -> Result<()> {
    let data = serde_json::to_vec(conf)?;
    storage::write(CONF_LOCAL_FILE, data).await?;
    Ok(())
}

async fn read_conf() -> Result<EndpointConf> {
    let data = storage::read(CONF_LOCAL_FILE).await?;
    let conf: EndpointConf = serde_json::from_slice(&data)?;
    Ok(conf)
}

async fn exist_conf() -> Result<bool> {
    let exist = storage::is_exist(CONF_LOCAL_FILE).await?;
    Ok(exist)
}
