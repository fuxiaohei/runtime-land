use super::{TaskTotal, TaskValue};
use crate::agent::{CLIENT, DATA_DIR};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tracing::{debug, info};

async fn download_wasm(data_dir: &str, task: &TaskValue) -> Result<()> {
    let wasm_path = format!("{}/{}", data_dir, task.wasm_path);
    if !PathBuf::from(wasm_path.clone()).exists() {
        // 1.1 download wasm from url
        let resp = CLIENT.get(&task.download_url).send().await?;
        let wasm_bytes = resp.bytes().await?;
        // 1.2 save wasm to file
        let wasm_dir = PathBuf::from(wasm_path.clone());
        let wasm_dir = wasm_dir.parent().unwrap();
        std::fs::create_dir_all(wasm_dir)?;
        info!("Save wasm to file: {}", wasm_path);
        std::fs::write(wasm_path, wasm_bytes)?;
    } else {
        debug!("Wasm file already exists: {}", wasm_path);
    }
    Ok(())
}

fn write_traefik_conf(data_dir: &str, task: &TaskValue) -> Result<()> {
    let conf_file = format!(
        "{}/traefik/{}.yaml",
        data_dir,
        task.domain.replace('.', "_")
    );
    let conf_dir = PathBuf::from(conf_file.clone());
    let conf_dir = conf_dir.parent().unwrap();
    std::fs::create_dir_all(conf_dir)?;

    let traefik_confs = build_item(task)?;
    let traefik_yaml = serde_yaml::to_string(&traefik_confs)?;
    info!("Write traefik conf to file: {}", conf_file);
    std::fs::write(conf_file, traefik_yaml)?;
    Ok(())
}

/// handle_task handles the task
pub async fn handle_task(task: &TaskValue) -> Result<()> {
    // 1. save wasm to file
    let data_dir = DATA_DIR.lock().await;
    download_wasm(data_dir.as_str(), task).await?;

    // 2. load wasm into mem
    land_wasm::pool::prepare_worker(&task.wasm_path, true).await?;

    // 3. write traefik conf after wasm is ready
    write_traefik_conf(data_dir.as_str(), task)?;

    Ok(())
}

/// handle_total handles the total tasks
pub async fn handle_total(data_dir: &str, total: &TaskTotal) -> Result<()> {
    for task in &total.tasks {
        // 1. it downloads wasm module file
        download_wasm(data_dir, task).await?;

        // 2. not load wasm to memory, generate aot wasm to make sure loading quickly
        let wasm_path = format!("{}/{}", data_dir, task.wasm_path);
        land_wasm::pool::compile_aot(&wasm_path).await?;

        // 3. write traefik conf
        write_traefik_conf(data_dir, task)?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraefikConfs {
    pub http: HttpTraefikConfs,
}

fn build_item(item: &TaskValue) -> Result<TraefikConfs> {
    let mut traefik_confs = HttpTraefikConfs {
        //services: HashMap::new(),
        routers: HashMap::new(),
        middlewares: HashMap::new(),
    };
    let svc = std::env::var("LAND_SERVICE_NAME").unwrap_or_else(|_| "runtimeland-foo".to_string());
    let mut headers = MiddlewareHeader {
        custom_request_headers: HashMap::new(),
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
