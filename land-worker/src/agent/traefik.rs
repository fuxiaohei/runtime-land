use anyhow::Result;
use land_kernel::cron::ConfData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub custom_request_headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareGroup {
    pub headers: MiddlewareHeader,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpTraefikConfs {
    // pub services: HashMap<String, Service>,
    pub middlewares: HashMap<String, MiddlewareGroup>,
    pub routers: HashMap<String, Router>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraefikConfs {
    pub http: HttpTraefikConfs,
}

/// build traefik configuration from ConfData
pub fn build(data: &ConfData) -> Result<TraefikConfs> {
    let mut traefik_confs = HttpTraefikConfs {
        //services: HashMap::new(),
        routers: HashMap::new(),
        middlewares: HashMap::new(),
    };
    let svc = std::env::var("LAND_SERVICE_NAME").unwrap_or_else(|_| "runtimeland-foo".to_string());
    for item in data.items.iter() {
        let mut headers = MiddlewareHeader {
            custom_request_headers: HashMap::new(),
        };
        // check filepath exist
        headers
            .custom_request_headers
            .insert("x-land-module".to_string(), item.path.to_string());
        headers
            .custom_request_headers
            .insert("x-land-user-id".to_string(), item.user_id.to_string());
        headers
            .custom_request_headers
            .insert("x-land-project-id".to_string(), item.project_id.to_string());
        traefik_confs
            .middlewares
            .insert(format!("m-{}", item.key), MiddlewareGroup { headers });

        let router = Router {
            middlewares: vec![format!("m-{}", item.key)],
            service: svc.clone(),
            rule: format!("Host(`{}`)", item.domain),
        };
        traefik_confs
            .routers
            .insert(format!("r-{}", item.key), router);
    }

    Ok(TraefikConfs {
        http: traefik_confs,
    })
}

/// build_yaml will build traefik configuration and convert it to yaml
pub async fn build_data_yaml(data: &ConfData) -> Result<String> {
    let confs = build(data)?;
    let content = serde_yaml::to_string(&confs)?;
    Ok(content)
}
