use anyhow::Result;
use axum::routing::get;
use axum::Router;
use land_api_server::{ConfData, SyncRequest, SyncResponse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

/// CONFS is the global confs data
pub static CONFS: Lazy<Mutex<ConfData>> = Lazy::new(|| {
    let op = ConfData {
        routes_md5: "".to_string(),
        routes: vec![],
        created_at: 0,
    };
    Mutex::new(op)
});

struct RunnerSyncRequest {
    token: String,
    cloud_url: String,
    local_url: String,
    ipinfo: land_common::IpInfo,
}

pub fn init_loop(token: String, cloud_url: String, local_url: String) -> Result<()> {
    debug!("init_loop, url: {}", cloud_url);

    let ipinfo = land_common::get_ip_info()?;
    info!("ipinfo: {:?}", ipinfo);

    tokio::spawn(async move {
        let req = RunnerSyncRequest {
            token,
            cloud_url,
            local_url,
            ipinfo,
        };
        // run loop_once in background and every 1 second
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if let Err(err) = sync_once(&req).await {
                error!("confs loop_once error: {:?}", err);
            }
        }
    });

    Ok(())
}

async fn sync_once(req: &RunnerSyncRequest) -> Result<()> {
    let url = format!("{}/api/v2/runner/sync", req.cloud_url);
    let mut current_conf = CONFS.lock().await;
    let req_data = SyncRequest {
        runner_token: req.token.clone(),
        confs_md5: current_conf.routes_md5.clone(),
        ipinfo: req.ipinfo.clone(),
    };
    let res: SyncResponse = ureq::post(&url).send_json(req_data)?.into_json()?;
    if res.is_modified {
        *current_conf = res.confs.unwrap();
        info!("sync_once, confs updated, md5: {}", current_conf.routes_md5);
        update_traefik_confs(current_conf.clone(), req.local_url.clone()).await;
    } else {
        debug!("sync_once, confs not modified");
    }
    Ok(())
}

async fn confs_traefik_handler() -> String {
    let traefik_confs = TRAEFIK_CONFS.lock().await;
    serde_json::to_string(&*traefik_confs).unwrap()
}

pub async fn start_server(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/confs/traefik", get(confs_traefik_handler))
        .layer(TraceLayer::new_for_http());
    info!("Starting confs server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServiceLoadBalancerServer {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServiceLoadBalancer {
    pub servers: Vec<HttpServiceLoadBalancerServer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpService {
    #[serde(rename = "loadBalancer")]
    pub load_balancer: HttpServiceLoadBalancer,
}

#[derive(Serialize, Deserialize)]
pub struct HttpRouter {
    #[serde(rename = "entryPoints")]
    pub entry_points: Vec<String>,
    pub middlewares: Vec<String>,
    pub service: String,
    pub rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpMiddlewareHeader {
    // #[serde(rename = "customResponseHeaders")]
    // pub custom_response_headers: HashMap<String, String>,
    #[serde(rename = "customRequestHeaders")]
    pub custom_request_headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpMiddlewareGroup {
    pub headers: HttpMiddlewareHeader,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpTraefikConfs {
    pub services: HashMap<String, HttpService>,
    pub middlewares: HashMap<String, HttpMiddlewareGroup>,
    // pub routers: HashMap<String, HttpRouter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraefikConfs {
    pub http: HttpTraefikConfs,
}

/// TRAEFIK_CONFS is the global traefik confs data
static TRAEFIK_CONFS: Lazy<Mutex<TraefikConfs>> = Lazy::new(|| {
    let http_confs = HttpTraefikConfs {
        services: HashMap::new(),
        middlewares: HashMap::new(),
        // routers: HashMap::new(),
    };
    Mutex::new(TraefikConfs { http: http_confs })
});

pub async fn update_traefik_confs(confs: ConfData, local_url: String) {
    let mut traefik_confs = HttpTraefikConfs {
        services: HashMap::new(),
        // routers: HashMap::new(),
        middlewares: HashMap::new(),
    };

    // add server as self service
    let lb = HttpServiceLoadBalancer {
        servers: vec![HttpServiceLoadBalancerServer { url: local_url }],
    };
    let service = HttpService { load_balancer: lb };
    traefik_confs
        .services
        .insert("rt-land-runner".to_string(), service);

    for route in confs.routes {
        let mut headers = HttpMiddlewareHeader {
            // custom_response_headers: HashMap::new(),
            custom_request_headers: HashMap::new(),
        };
        headers
            .custom_request_headers
            .insert("x-land-uuid".to_string(), route.uuid.clone());
        headers
            .custom_request_headers
            .insert("x-land-module".to_string(), route.module_path.clone());
        headers
            .custom_request_headers
            .insert("x-land-project".to_string(), route.project_id.to_string());
        headers
            .custom_request_headers
            .insert("x-land-user".to_string(), route.owner_id.to_string());
        traefik_confs
            .middlewares
            .insert(format!("m-{}", route.uuid), HttpMiddlewareGroup { headers });
    }
    let mut current_traefik_confs = TRAEFIK_CONFS.lock().await;
    *current_traefik_confs = TraefikConfs {
        http: traefik_confs,
    };
    debug!("update_traefik_confs, confs: {:?}", current_traefik_confs);
}
