use anyhow::Result;
use axum::routing::get;
use axum::Router;
use land_api_server::{ConfData, SyncRequest, SyncResponse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
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
        update_traefik_confs(current_conf.clone()).await;
    } else {
        debug!("sync_once, confs not modified");
    }
    Ok(())
}

async fn confs_traefik_handler() -> String {
    let traefik_confs = TRAEFIK_CONFS.lock().await;
    toml::to_string(&*traefik_confs).unwrap()
}

pub async fn start_server(addr: SocketAddr) -> Result<()> {
    let app = Router::new().route("/confs/traefik", get(confs_traefik_handler));
    info!("Starting confs server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct HttpServiceLoadBalancerServer {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct HttpServiceLoadBalancer {
    pub servers: Vec<HttpServiceLoadBalancerServer>,
}

#[derive(Serialize, Deserialize)]
pub struct HttpRouter {
    #[serde(rename = "entryPoints")]
    pub entry_points: Vec<String>,
    pub middlewares: Vec<String>,
    pub service: String,
    pub rule: String,
}

#[derive(Serialize, Deserialize)]
pub struct HttpMiddlewareHeader {
    #[serde(rename = "customResponseHeaders")]
    pub custom_response_headers: HashMap<String, String>,
    #[serde(rename = "customRequestHeaders")]
    pub custom_request_headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct HttpMiddlewareGroup {
    pub headers: HttpMiddlewareHeader,
}

#[derive(Serialize, Deserialize)]
pub struct HttpTraefikConfs {
    pub services: HashMap<String, HttpServiceLoadBalancer>,
    pub routers: HashMap<String, HttpRouter>,
    pub middlewares: HashMap<String, HttpMiddlewareGroup>,
}

#[derive(Serialize, Deserialize)]
pub struct TraefikConfs {
    pub http: HttpTraefikConfs,
}

/// TRAEFIK_CONFS is the global traefik confs data
static TRAEFIK_CONFS: Lazy<Mutex<TraefikConfs>> = Lazy::new(|| {
    let http_confs = HttpTraefikConfs {
        services: HashMap::new(),
        routers: HashMap::new(),
        middlewares: HashMap::new(),
    };
    Mutex::new(TraefikConfs { http: http_confs })
});

pub async fn update_traefik_confs(confs: ConfData) {
    let mut traefik_confs = HttpTraefikConfs {
        services: HashMap::new(),
        routers: HashMap::new(),
        middlewares: HashMap::new(),
    };
    for route in confs.routes {
        let mut headers = HttpMiddlewareHeader {
            custom_response_headers: HashMap::new(),
            custom_request_headers: HashMap::new(),
        };
        headers
            .custom_request_headers
            .insert("x-land-uuid".to_string(), route.uuid.clone());
        headers
            .custom_request_headers
            .insert("x-land-module".to_string(), route.resource_path.clone());
        traefik_confs.middlewares.insert(
            format!("middleware-{}", route.uuid),
            HttpMiddlewareGroup { headers },
        );
    }
    let mut current_traefik_confs = TRAEFIK_CONFS.lock().await;
    *current_traefik_confs = TraefikConfs {
        http: traefik_confs,
    };
}
