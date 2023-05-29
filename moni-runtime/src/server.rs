use crate::host_call::http_incoming::http_incoming::Request as WasmRequest;
use crate::Context;
use crate::{create_pool, WorkerPool};
use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use lazy_static::lazy_static;
use moka::sync::Cache;
use moni_lib::storage::STORAGE;
use once_cell::sync::OnceCell;
use std::net::SocketAddr;
use std::{sync::Arc, time::Duration};
use tracing::{debug, info, info_span, warn, Instrument};

lazy_static! {
    pub static ref WASM_INSTANCES: Cache<String,Arc<WorkerPool> > = Cache::builder()
    // Time to live (TTL): 1 hours
    .time_to_live(Duration::from_secs( 60 * 60))
    // Time to idle (TTI):  10 minutes
    .time_to_idle(Duration::from_secs(10 * 60))
    // Create the cache.
    .build();
}

// DEFAULT_WASM_PATH is used to set default wasm path
pub static DEFAULT_WASM_PATH: OnceCell<String> = OnceCell::new();

pub async fn prepare_worker_pool(key: &str) -> Result<Arc<WorkerPool>> {
    let mut instances_pool = WASM_INSTANCES.get(key);

    if instances_pool.is_some() {
        return Ok(instances_pool.unwrap());
    }

    let storage = STORAGE.get().expect("storage is not initialized");
    if !storage.is_exist(key).await? {
        return Err(anyhow!("key not found: {}", key));
    }
    let binary = storage.read(key).await?;

    // write binary to local file
    let mut path = std::env::temp_dir();
    path.push(key);
    // create parent dir
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    std::fs::write(&path, binary)?;
    debug!("wasm temp binary write to {}", path.display());

    // create wasm worker pool
    let pool = create_pool(path.to_str().unwrap())?;
    WASM_INSTANCES.insert(key.to_string(), Arc::new(pool));

    instances_pool = WASM_INSTANCES.get(key);

    Ok(instances_pool.unwrap())
}

pub async fn wasm_caller_handler(
    req: Request<Body>,
    moni_wasm: &str,
    req_id: String,
) -> Result<Response<Body>> {
    let pool = prepare_worker_pool(moni_wasm).await?;
    let mut worker = pool.get().await.map_err(|e| anyhow!(e.to_string()))?;
    debug!("[HTTP] wasm worker pool get worker success: {}", moni_wasm);

    // convert request to host-call request
    let mut headers: Vec<(String, String)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.to_string(), v.to_str().unwrap().to_string()));
    });

    let uri = req.uri().to_string();
    let method = req.method().clone();
    let mut context = Context::new(req_id);
    let req_id = context.req_id();
    let body = req.into_body();
    let body_handle = context.set_body(body);
    let wasm_req = WasmRequest {
        method: method.as_str(),
        uri: uri.as_str(),
        headers: &headers,
        body: Some(body_handle),
    };

    let span = info_span!("[WASM]", moni_wasm = %moni_wasm, body = ?body_handle);
    let _enter = span.enter();

    let (wasm_resp, wasm_resp_body) = match worker.handle_request(wasm_req, context).await {
        Ok((wasm_resp, wasm_resp_body)) => (wasm_resp, wasm_resp_body),
        Err(e) => {
            let builder = Response::builder().status(500);
            return Ok(builder.body(Body::from(e.to_string())).unwrap());
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
    if wasm_resp.status >= 400 {
        warn!( status=%wasm_resp.status, "[Response]");
    } else {
        info!( status=%wasm_resp.status, "[Response]");
    }
    Ok(builder.body(wasm_resp_body).unwrap())
}

// basic handler that responds with a static string
async fn default_handler(req: Request<Body>) -> Response<Body> {
    let req_id = uuid::Uuid::new_v4().to_string();
    // get header moni-wasm
    let headers = req.headers().clone();
    let empty_wasm_path = String::new();
    let moni_wasm = headers
        .get("moni-wasm")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(DEFAULT_WASM_PATH.get().unwrap_or(&empty_wasm_path));

    let method = req.method().clone();
    let uri = req.uri().to_string();
    let span = info_span!("[HTTP]",req_id = %req_id.clone(), method = %method, uri = %uri,);

    if moni_wasm.is_empty() {
        let _enter = span.enter();
        let builder = Response::builder().status(404);
        warn!(status = 404, "[Response] moni-wasm not found");
        return builder.body(Body::from("moni-wasm not found")).unwrap();
    }

    match wasm_caller_handler(req, moni_wasm, req_id)
        .instrument(span)
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let builder = Response::builder().status(500);
            builder.body(Body::from(e.to_string())).unwrap()
        }
    }
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
