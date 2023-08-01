use anyhow::{anyhow, Result};
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use deadpool::managed;
use land_core::metadata::Metadata;
use land_worker::hostcall::Request as WasmRequest;
use land_worker::Worker;
use once_cell::sync::OnceCell;
use std::net::SocketAddr;
use tokio::time::Instant;
use tracing::{debug, debug_span, info, warn};

/// WASM_POOLER is a global worker pooler
static WASM_POOLER: OnceCell<WorkerPooler> = OnceCell::new();

#[derive(Debug)]
struct Pooler {
    path: String,
}

impl Pooler {
    pub fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
        }
    }
}

#[async_trait]
impl managed::Manager for Pooler {
    type Type = Worker;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let start_time = Instant::now();
        let worker = Worker::new(&self.path).await?;
        debug_span!("[Worker]", path = &self.path).in_scope(|| {
            debug!(eplased = ?start_time.elapsed(), "create, ok");
        });
        Ok(worker)
    }

    async fn recycle(&self, _obj: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

/// WorkerPooler is a worker pooler
type WorkerPooler = managed::Pool<Pooler>;

async fn default_handler(req: Request<Body>) -> Response<Body> {
    let now = tokio::time::Instant::now();

    let pooler = WASM_POOLER.get().unwrap();
    let mut worker = pooler
        .get()
        .await
        .map_err(|e| anyhow!(e.to_string()))
        .unwrap();

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

    let pooler = Pooler::new(&output);
    let worker_pooler = managed::Pool::builder(pooler).build().unwrap();
    WASM_POOLER.set(worker_pooler).unwrap();

    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
