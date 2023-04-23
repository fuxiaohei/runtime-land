use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use moni_lib::meta::Meta;
use moni_runtime::host_call::http_incoming::http_incoming::Request as WasmRequest;
use moni_runtime::{Context, WorkerPool};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::OnceCell;
use tracing::info;

/// WASM_POOL is a global wasm worker pool
static WASM_POOL: OnceCell<WorkerPool> = OnceCell::const_new();

/// GLOBAL_REQUEST_COUNT is a global request count
static GLOBAL_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1);

/// start server
pub async fn start(addr: SocketAddr, meta: &Meta) -> Result<()> {
    // init global wasm worker pool
    let pool = moni_runtime::create_pool(&meta.get_output())?;
    WASM_POOL.set(pool).unwrap();
    info!("wasm pool created");

    // build our application with default handler
    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn default_handler(req: Request<Body>) -> Response<Body> {
    let pool = WASM_POOL.get().expect("wasm pool not found");
    let mut worker = pool.get().await.expect("wasm worker not found");

    // convert request to host-call request
    let mut headers: Vec<(String, String)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.to_string(), v.to_str().unwrap().to_string()));
    });

    let uri = req.uri().to_string();
    let method = req.method().clone();

    // call worker execute
    let req_id = GLOBAL_REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
    let mut context = Context::new(req_id);
    let body = req.into_body();
    let body_handle = context.set_body(body);

    let wasm_req = WasmRequest {
        method: method.as_str(),
        uri: uri.as_str(),
        headers: &headers,
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
    for (k, v) in wasm_resp.headers {
        builder = builder.header(k, v);
    }
    builder.body(wasm_resp_body).unwrap()
}
