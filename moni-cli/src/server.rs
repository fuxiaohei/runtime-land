use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use moni_runtime::host_call::http_incoming::RequestParam;
use moni_runtime::{Context, WorkerPool};
use std::net::SocketAddr;
use tokio::sync::OnceCell;
use tracing::info;

/// WASM_POOL is a global wasm worker pool
static WASM_POOL: OnceCell<WorkerPool> = OnceCell::const_new();

/// start server
pub async fn start(addr: SocketAddr) -> Result<()> {
    // init global wasm worker pool
    let pool = moni_runtime::create_pool("./tests/data/rust_impl.component.wasm")?;
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
    let mut headers: Vec<(&str, &str)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.as_str(), v.to_str().unwrap()));
    });

    let uri = req.uri().to_string();
    let method = req.method().clone();

    // call worker execute
    let mut context = Context::new();
    let body = req.into_body();
    let body_handle = context.set_body(body);
    let wasm_req = RequestParam {
        method: method.as_str(),
        uri: uri.as_str(),
        headers: &headers,
        body: Some(body_handle),
    };
    println!("----- wasm_req = {:?}", wasm_req);

    let wasm_resp = worker.handle_request(wasm_req, context).await.unwrap();

    Response::builder()
        .status(wasm_resp.status)
        .body(Body::from("Hello, World!"))
        .unwrap()
}
