use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use land_worker::hostcall::Request as WasmRequest;
use land_worker::Context;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{debug, info, info_span, warn, Instrument};

pub async fn wasm_caller_handler(
    req: Request<Body>,
    wasm_path: &str,
    req_id: String,
) -> Result<Response<Body>> {
    let pool = crate::pool::prepare_worker_pool(wasm_path)
        .instrument(info_span!("[WASM]", wasm_path = %wasm_path))
        .await?;
    let mut worker = pool.get().await.map_err(|e| anyhow!(e.to_string()))?;
    debug!("[HTTP] wasm worker pool get worker success: {}", wasm_path);

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
        method: method.to_string(),
        uri,
        headers,
        body: Some(body_handle),
    };

    let span = info_span!("[WASM]", wasm_path = %wasm_path, body = ?body_handle);
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
    // get header x-land-wasm
    let headers = req.headers().clone();
    let empty_wasm_path = String::new();
    let land_wasm = headers
        .get("x-land-wasm")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(
            crate::pool::DEFAULT_WASM_PATH
                .get()
                .unwrap_or(&empty_wasm_path),
        );

    let method = req.method().clone();
    let uri = req.uri().to_string();
    let span = info_span!("[HTTP]",req_id = %req_id.clone(), method = %method, uri = %uri,);

    if land_wasm.is_empty() {
        let _enter = span.enter();
        let mut builder = Response::builder().status(404);
        builder = builder.header("x-request-id", req_id);
        warn!(status = 404, "[Response] x-land-wasm not found");
        return builder.body(Body::from("x-land-wasm not found")).unwrap();
    }

    match wasm_caller_handler(req, land_wasm, req_id.clone())
        .instrument(span)
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            warn!(status = 500, "[Response] {}", e.to_string());
            let mut builder = Response::builder().status(500);
            builder = builder.header("x-request-id", req_id);
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
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutting down");
}
