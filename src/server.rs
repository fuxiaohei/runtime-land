use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use land_core::metadata::Metadata;
use land_worker::hostcall::Request as WasmRequest;
use land_worker::Worker;
use lazy_static::lazy_static;
use moka::sync::Cache;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{info, warn};

lazy_static! {
    pub static ref INSTANCES_CACHE: Cache<String,Worker> = Cache::builder()
    // Time to live (TTL): 3 hours
    .time_to_live(Duration::from_secs( 3*60 * 60))
    // Time to idle (TTI):  1 hour
    .time_to_idle(Duration::from_secs(60 * 60))
    // Create the cache.
    .build();
}

async fn default_handler(req: Request<Body>) -> Response<Body> {
    let now = tokio::time::Instant::now();

    let worker = INSTANCES_CACHE.get("default").unwrap();

    let req_id = uuid::Uuid::new_v4().to_string();
    let mut headers: Vec<(String, String)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.to_string(), v.to_str().unwrap().to_string()));
    });

    let uri = req.uri().to_string();
    let method = req.method().clone();
    let mut context = land_worker::Context::new(req_id.clone());

    let body = req.into_body();
    let body_handle = context.set_body(body);
    let wasm_req = WasmRequest {
        method: method.to_string(),
        uri,
        headers,
        body: Some(body_handle),
    };

    let (wasm_resp, wasm_resp_body) = match worker.handle_request(wasm_req, context).await {
        Ok((wasm_resp, wasm_resp_body)) => (wasm_resp, wasm_resp_body),
        Err(e) => {
            let builder = Response::builder().status(500);
            return builder.body(Body::from(e.to_string())).unwrap();
        }
    };

    // convert host-call response to response
    let mut builder = Response::builder().status(wasm_resp.status);
    for (k, v) in wasm_resp.headers.clone() {
        builder = builder.header(k, v);
    }
    if builder.headers_ref().unwrap().get("x-request-id").is_none() {
        builder = builder.header("x-request-id", req_id.clone());
    }
    let elapsed = now.elapsed();
    if wasm_resp.status >= 400 {
        warn!(elapsed = ?elapsed,req_id=%req_id, status=%wasm_resp.status, "[Response]");
    } else {
        info!(elapsed = ?elapsed,req_id=%req_id, status=%wasm_resp.status, "[Response]");
    }
    builder.body(wasm_resp_body).unwrap()
}

pub async fn start(addr: SocketAddr, meta: &Metadata) -> Result<()> {
    let output = meta.get_output();
    info!("WASM_PATH: {}", output);

    let worker = Worker::new(&output).await?;
    INSTANCES_CACHE.insert("default".to_string(), worker);

    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
