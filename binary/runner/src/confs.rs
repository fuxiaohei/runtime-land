use anyhow::Result;
use land_api_server::{ConfData, SyncRequest, SyncResponse};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// CONFS is the global confs data
pub static CONFS: Lazy<Mutex<ConfData>> = Lazy::new(|| {
    let op = ConfData {
        routes_md5: "".to_string(),
        routes: vec![],
    };
    Mutex::new(op)
});

pub fn init_loop(token: String, cloud_url: String) -> Result<()> {
    debug!("init_loop, url: {}", cloud_url);

    tokio::spawn(async move {
        // run loop_once in background and every 1 second
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if let Err(err) = sync_once(token.clone(), cloud_url.clone()).await {
                error!("confs loop_once error: {:?}", err);
            }
        }
    });

    Ok(())
}

async fn sync_once(token: String, cloud_url: String) -> Result<()> {
    let url = format!("{}/api/v2/runner/sync", cloud_url);
    let mut current_conf = CONFS.lock().await;
    let req_data = SyncRequest {
        runner_token: token.to_string(),
        confs_md5: current_conf.routes_md5.clone(),
    };
    let res: SyncResponse = ureq::post(&url).send_json(req_data)?.into_json()?;
    if res.is_modified {
        *current_conf = res.confs.unwrap();
        info!("sync_once, confs updated, md5: {}", current_conf.routes_md5);
    }else{
        debug!("sync_once, confs not modified");
    }
    Ok(())
}
