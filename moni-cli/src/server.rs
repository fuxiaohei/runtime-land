use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::Result;
use hyper::StatusCode;
use moni_core::keyvalue::SledStorage;
use moni_core::Meta;
use moni_runtime::http_impl::http_handler::{Request as HostRequest, Response as HostResponse};
use moni_runtime::kv_impl::Provider;
use moni_runtime::{Context, WorkerPool};
use once_cell::sync::{Lazy, OnceCell};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// WASM_POOL is a global wasm worker pool
static WASM_POOL: OnceCell<WorkerPool> = OnceCell::new();

/// KV_STORAGE is global kv storage for local cli
static KV_STORAGE: Lazy<Provider> =
    Lazy::new(|| Arc::new(Mutex::new(SledStorage::new(&get_default_kv_db()).unwrap())));

/// get_default_kv_db returns default kv db path
pub fn get_default_kv_db() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home)
        .join(".moni_keyvalue_db")
        .to_str()
        .unwrap()
        .to_string()
}

/// start server
pub async fn start(addr: SocketAddr, meta: &Meta) -> Result<()> {
    // init global wasm worker pool
    let pool = moni_runtime::create_pool(&meta.get_output())?;
    WASM_POOL.set(pool).unwrap();
    info!("wasm pool created");

    // start actix-web http server
    info!("starting on {}", addr);
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .default_service(web::to(default_handler))
    })
    .bind(&addr)?
    .run()
    .await?;
    Ok(())
}

async fn default_handler(req: HttpRequest, req_body_bytes: web::Bytes) -> HttpResponse {
    let pool = WASM_POOL.get().expect("wasm pool not found");
    let mut worker = pool.get().await.expect("wasm worker not found");

    // convert request to host-call request
    let mut headers: Vec<(&str, &str)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.as_str(), v.to_str().unwrap()));
    });

    let url = req.uri().to_string();
    let method = req.method().clone();
    let body_bytes = req_body_bytes.to_vec();

    let host_req = HostRequest {
        method: method.as_str(),
        uri: url.as_str(),
        headers: &headers,
        body: Some(&body_bytes),
    };

    // create runtime context
    let mut context = Context::new();
    context.set_kv_provider(KV_STORAGE.clone());

    // call worker execute
    let host_resp: HostResponse = worker.handle_request(host_req, context).await.unwrap();

    // convert host-call response to axum response
    let mut builder = HttpResponse::build(StatusCode::from_u16(host_resp.status).unwrap());
    for (k, v) in host_resp.headers {
        builder.insert_header((k.as_str(), v.as_str()));
    }
    builder.body(host_resp.body.unwrap())
}
