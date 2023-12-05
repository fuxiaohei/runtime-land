use anyhow::Result;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode};
use axum::routing::any;
use axum::Router;
use land_worker::{Context, WasmRequest};
use once_cell::sync::OnceCell;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::Instant;
use tower_http::timeout::TimeoutLayer;
use tracing::{debug, info, info_span, warn, Instrument};

mod fs;
mod pool;

static DEFAULT_WASM: OnceCell<String> = OnceCell::new();

async fn default_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
) -> Response<Body> {
    let st = Instant::now();
    let req_id = uuid::Uuid::new_v4().to_string();

    // get header x-land-module
    let headers = req.headers().clone();
    let default_wasm_path = DEFAULT_WASM.get().unwrap();
    let land_wasm = headers
        .get("x-land-module")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(default_wasm_path.as_str());

    let method = req.method().clone();
    let uri = req.uri().to_string();
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let span = info_span!("[HTTP]",remote = %addr.to_string(), req_id = %req_id.clone(), method = %method, uri = %uri, host = %host);
    let span_clone = span.clone();

    if land_wasm.is_empty() {
        let _enter = span.enter();
        let mut builder = Response::builder().status(StatusCode::NOT_FOUND);
        builder = builder.header("x-request-id", req_id);
        builder = builder.header("x-served-by", "abc");
        let elapsed = st.elapsed().as_micros();
        warn!(
            status = 404,
            elapsed = %elapsed,
            "x-land-module not found",
        );
        return builder.body(Body::from("x-land-module not found")).unwrap();
    }

    async move {
        let result = wasm_caller_handler(req, land_wasm, req_id.clone())
            .instrument(span)
            .await;
        if result.is_err() {
            let e = result.err().unwrap();
            let mut builder = Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR);
            builder = builder.header("x-request-id", req_id);
            builder = builder.header("x-served-by", "abc");
            let elapsed = st.elapsed().as_micros();
            warn!(
                status = 500,
                elapsed = %elapsed,
                "internal error: {}",
                e.to_string(),
            );
            return builder.body(Body::from(e.to_string())).unwrap();
        }
        let resp = result.unwrap();
        let status_code = resp.status().as_u16();
        let elapsed = st.elapsed().as_micros();
        if status_code >= 400 {
            warn!( status=%status_code,elapsed=%elapsed, "Done");
        } else {
            info!( status=%status_code,elapsed=%elapsed, "Done");
        }
        resp
    }
    .instrument(span_clone)
    .await
}

pub async fn wasm_caller_handler(
    req: Request<Body>,
    wasm_path: &str,
    req_id: String,
) -> Result<Response<Body>> {
    let worker = pool::prepare_worker(wasm_path)
        .instrument(info_span!("[WASM]", wasm_path = %wasm_path))
        .await?;
    debug!("Wasm worker pool ok: {}", wasm_path);

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
    builder = builder.header("x-served-by", "abc");
    Ok(builder.body(wasm_resp_body).unwrap())
}

async fn start_server(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler))
        .layer(TimeoutLayer::new(Duration::from_secs(10)));
    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();

    info!("Starting on {}", addr);

    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, make_service).await?;
    Ok(())
}

pub struct Opts {
    pub addr: SocketAddr,
    pub dir: String,
    pub default_wasm: String,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:10000".parse().unwrap(),
            dir: "/tmp/land".to_string(),
            default_wasm: "".to_string(),
        }
    }
}

/// run starts the server
pub async fn run(opts: Opts) -> Result<()> {
    // init local fs to read wasm files
    fs::init_fs(&opts.dir)?;

    // set default wasm, can be empty
    DEFAULT_WASM.set(opts.default_wasm).unwrap();

    // start server
    start_server(opts.addr).await
}
