use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use moni_runtime::host_call::http_incoming::http_incoming::Request as WasmRequest;
use moni_runtime::Context;
use std::net::SocketAddr;
use tracing::info;

mod rt;

async fn wasm_caller_handler(req: Request<Body>, moni_wasm: &str) -> Result<Response<Body>> {
    let pool = rt::prepare_worker_pool(moni_wasm).await?;
    let mut worker = pool.get().await.map_err(|e| anyhow!(e.to_string()))?;

    // convert request to host-call request
    let mut headers: Vec<(String, String)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.to_string(), v.to_str().unwrap().to_string()));
    });

    let uri = req.uri().to_string();
    let method = req.method().clone();
    let mut context = Context::new(1);
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
            return Ok(builder.body(Body::from(e.to_string())).unwrap());
        }
    };

    // convert host-call response to response
    let mut builder = Response::builder().status(wasm_resp.status);
    for (k, v) in wasm_resp.headers {
        builder = builder.header(k, v);
    }
    Ok(builder.body(wasm_resp_body).unwrap())
}

// basic handler that responds with a static string
async fn default_handler(req: Request<Body>) -> Response<Body> {
    // get header moni-wasm
    let headers = req.headers().clone();
    let moni_wasm = headers
        .get("moni-wasm")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if moni_wasm.is_empty() {
        let builder = Response::builder().status(404);
        return builder.body(Body::from("moni-wasm not found")).unwrap();
    }

    match wasm_caller_handler(req, moni_wasm).await {
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

    info!("starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
