use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
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

    // build our application with a route
    let app = Router::new()
        .route("/", any(root))
        .route("/*path", any(root));

    info!("starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn root(req: Request<Body>) -> Response<Body> {
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
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .unwrap()
        .to_vec();

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
    let mut builder = Response::builder().status(host_resp.status);
    for (k, v) in host_resp.headers {
        builder = builder.header(k, v);
    }
    builder.body(Body::from(host_resp.body.unwrap())).unwrap()
}
