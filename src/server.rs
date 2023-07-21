use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use land_core::metadata::Metadata;
use land_worker::hostcall::Request as WasmRequest;
use once_cell::sync::OnceCell;
use std::net::SocketAddr;
use tracing::{info, warn};

/// WASM_PATH is a global worker path
static WASM_PATH: OnceCell<String> = OnceCell::new();

async fn default_handler(req: Request<Body>) -> Response<Body> {
    let wasm_path = WASM_PATH.get().unwrap();
    let mut worker = land_worker::Worker::new(&wasm_path).await.unwrap();

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
    if wasm_resp.status >= 400 {
        warn!( status=%wasm_resp.status, "[Response]");
    } else {
        info!( status=%wasm_resp.status, "[Response]");
    }
    builder.body(wasm_resp_body).unwrap()
}

pub async fn start(addr: SocketAddr, meta: &Metadata) -> Result<()> {
    let output = meta.get_output();
    WASM_PATH.set(output.clone()).unwrap();
    info!("WASM_PATH: {}", output);

    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
