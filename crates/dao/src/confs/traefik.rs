use super::TaskValue;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TraefikConfs {
    pub http: HttpTraefikConfs,
}

/// build_item builds the TraefikConfs for the given TaskValue.
pub fn build_item(item: &TaskValue) -> Result<TraefikConfs> {
    let mut traefik_confs = HttpTraefikConfs {
        //services: HashMap::new(),
        routers: BTreeMap::new(),
        middlewares: BTreeMap::new(),
    };
    let svc = std::env::var("LAND_SERVICE_NAME").unwrap_or_else(|_| "runtimeland-foo".to_string());
    let mut headers = MiddlewareHeader {
        custom_request_headers: BTreeMap::new(),
    };
    // check filepath exist
    headers
        .custom_request_headers
        .insert("x-land-m".to_string(), item.wasm_path.clone());
    headers
        .custom_request_headers
        .insert("x-land-uuid".to_string(), item.user_uuid.clone());
    headers
        .custom_request_headers
        .insert("x-land-puuid".to_string(), item.project_uuid.clone());
    traefik_confs
        .middlewares
        .insert(format!("m-{}", item.task_id), MiddlewareGroup { headers });

    let router = Router {
        middlewares: vec![format!("m-{}", item.task_id)],
        service: svc.clone(),
        rule: format!("Host(`{}`)", item.domain),
    };
    traefik_confs
        .routers
        .insert(format!("r-{}", item.task_id), router);
    Ok(TraefikConfs {
        http: traefik_confs,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceLoadBalancerServer {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceLoadBalancer {
    pub servers: Vec<ServiceLoadBalancerServer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    #[serde(rename = "loadBalancer")]
    pub load_balancer: ServiceLoadBalancer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Router {
    // #[serde(rename = "entryPoints")]
    // pub entry_points: Vec<String>,
    pub middlewares: Vec<String>,
    pub service: String,
    pub rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareHeader {
    // #[serde(rename = "customResponseHeaders")]
    // pub custom_response_headers: HashMap<String, String>,
    #[serde(rename = "customRequestHeaders")]
    pub custom_request_headers: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareGroup {
    pub headers: MiddlewareHeader,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpTraefikConfs {
    // pub services: HashMap<String, Service>,
    pub middlewares: BTreeMap<String, MiddlewareGroup>,
    pub routers: BTreeMap<String, Router>,
}
