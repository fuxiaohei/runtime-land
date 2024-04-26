use anyhow::Result;
use axum::body::HttpBody;
use axum::Extension;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::any,
    Router,
};
use land_wasm::hostcall::Request as WasmRequest;
use land_wasm::{Context, Worker};
use metrics_exporter_prometheus::PrometheusBuilder;
use once_cell::sync::OnceCell;
use std::{net::SocketAddr, time::Duration};
use tokio::time::Instant;
use tower_http::timeout::TimeoutLayer;
use tracing::{debug, info, info_span, warn, Instrument};

mod middleware;

/// Opts for the worker server
pub struct Opts {
    pub addr: SocketAddr,
    pub dir: String,
    pub default_wasm: String,
    pub endpoint_name: Option<String>,
    pub wasm_aot: bool,
    pub metrics: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:9844".parse().unwrap(),
            dir: "./data/land".to_string(),
            default_wasm: "".to_string(),
            endpoint_name: Some("localhost".to_string()),
            wasm_aot: false,
            metrics: false,
        }
    }
}

static ENDPOINT_NAME: OnceCell<String> = OnceCell::new();
static AOT_ENABLED: OnceCell<bool> = OnceCell::new();
static METRICS_ENABLED: OnceCell<bool> = OnceCell::new();

pub fn init_globals(opts: &Opts) -> Result<()> {
    let hostname = if let Some(endpoint) = &opts.endpoint_name {
        endpoint.clone()
    } else {
        hostname::get()
            .unwrap_or("unknown".into())
            .to_string_lossy()
            .to_string()
    };

    debug!("Endpoint: {}", hostname);
    debug!("Wasm dir: {}", opts.dir);
    debug!("Default wasm: {}", opts.default_wasm);
    debug!("AOT enabled: {}", opts.wasm_aot);
    debug!("Metrics enabled: {}", opts.metrics);

    // enable prometheus metrics api
    if opts.metrics {
        // use for local visit, :9000
        PrometheusBuilder::new().install()?;
        info!("Start metrics server: 127.0.0.1:9000");
    }

    // create directory
    std::fs::create_dir_all(&opts.dir).unwrap();

    DEFAULT_WASM.set(opts.default_wasm.clone()).unwrap();
    ENDPOINT_NAME.set(hostname).unwrap();
    AOT_ENABLED.set(opts.wasm_aot).unwrap();
    METRICS_ENABLED.set(opts.metrics).unwrap();

    // set pool's local dir to load module file
    land_wasm::pool::FILE_DIR.set(opts.dir.clone()).unwrap();

    // start wasmtime engines epoch calls
    land_wasm::hostcall::init_clients();
    land_wasm::init_engines()?;

    Ok(())
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    // load default wasm
    load_default_wasm().await?;
    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .route_layer(axum::middleware::from_fn(middleware::middleware));
    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();
    info!("Starting worker server on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, make_service).await?;
    Ok(())
}

static DEFAULT_WASM: OnceCell<String> = OnceCell::new();

pub async fn load_default_wasm() -> Result<()> {
    let default_wasm = DEFAULT_WASM.get().unwrap();
    if default_wasm.is_empty() {
        debug!("No default wasm");
        return Ok(());
    }
    let aot_enable = AOT_ENABLED.get().unwrap();
    let _ = land_wasm::pool::prepare_worker(default_wasm, *aot_enable).await?;
    Ok(())
}

async fn default_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(ctx): Extension<middleware::WorkerContext>,
    Extension(metrics): Extension<middleware::WorkerMetrics>,
    req: Request<Body>,
) -> Result<impl IntoResponse, ServerError> {
    let st = Instant::now();

    // prepare span info
    let method = req.method().clone();
    let uri = req.uri().to_string();

    let span = info_span!("[HTTP]",rt = %addr.to_string(), rid = %ctx.req_id.clone(), m = %method, u = %uri, h = %ctx.host);
    let span_clone = span.clone();
    metrics.req_fn_total.increment(1);

    // if wasm_module is empty, return 404
    if ctx.wasm_module.is_empty() {
        let _enter = span.enter();
        warn!(
            status = 404,
            elapsed = %st.elapsed().as_micros(),
            "Function not found",
        );
        metrics.req_fn_notfound_total.increment(1);
        return Err(ServerError::not_found(ctx, "Function not found"));
    }

    // collect post body size
    let body_size = req.body().size_hint().exact().unwrap_or(0);
    metrics.req_fn_in_bytes_total.increment(body_size);

    // call wasm async
    async move {
        let result = wasm_caller_handler(req, &ctx.wasm_module, ctx.req_id.clone()).await;
        if let Err(err) = result {
            let elapsed = st.elapsed().as_micros();
            warn!(
                status = 500,
                elapsed = %elapsed,
                "Internal error: {}",
                err,
            );
            metrics.req_fn_error_total.increment(1);
            let msg = format!("Internal error: {}", err);
            return Err(ServerError::internal_error(ctx, &msg));
        }
        let resp = result.unwrap();
        let status_code = resp.status().as_u16();
        let elapsed = st.elapsed().as_micros();
        if status_code >= 400 {
            warn!( status=%status_code,elapsed=%elapsed, "Done");
        } else {
            info!( status=%status_code,elapsed=%elapsed, "Done");
        }
        let body_size = resp.body().size_hint().exact().unwrap_or(0);
        metrics.req_fn_out_bytes_total.increment(body_size);
        Ok(resp)
    }
    .instrument(span_clone)
    .await
}

/// prepare_worker is a helper function to prepare wasm worker
pub async fn prepare_worker(wasm_path: &str) -> Result<Worker> {
    let aot_enable = AOT_ENABLED.get().unwrap();
    let worker = land_wasm::pool::prepare_worker(wasm_path, *aot_enable)
        .instrument(info_span!("[WASM]", wasm_path = %wasm_path))
        .await?;
    debug!("Wasm worker pool ok: {}", wasm_path);
    Ok(worker)
}

async fn wasm_caller_handler(
    req: Request<Body>,
    wasm_path: &str,
    req_id: String,
) -> Result<Response<Body>> {
    let worker = prepare_worker(wasm_path).await?;

    // convert request to host-call request
    let mut headers: Vec<(String, String)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        headers.push((k.to_string(), v.to_str().unwrap().to_string()));
    });

    let mut uri = req.uri().clone();
    // if no host, use host value to generate new one, must be full uri
    if uri.authority().is_none() {
        let host = req
            .headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let new_uri = format!("http://{}{}", host, uri.path());
        uri = new_uri.parse().unwrap();
    }
    let method = req.method().clone();
    let mut context = Context::new();
    // if method is GET or DELETE, set body to None
    let body_handle = if method == "GET" || method == "DELETE" {
        0
    } else {
        let body = req.into_body();
        context.set_body(0, body)
    };
    debug!("Set body_handle: {:?}", body_handle);
    let wasm_req = WasmRequest {
        method: method.to_string(),
        uri: uri.to_string(),
        headers,
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
    for (k, v) in wasm_resp.headers.clone() {
        builder = builder.header(k, v);
    }
    if builder.headers_ref().unwrap().get("x-request-id").is_none() {
        builder = builder.header("x-request-id", req_id.clone());
    }
    builder = builder.header("x-served-by", ENDPOINT_NAME.get().unwrap());
    Ok(builder.body(wasm_resp_body).unwrap())
}

pub struct ServerError(middleware::WorkerContext, StatusCode, anyhow::Error);

impl Clone for ServerError {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1, anyhow::anyhow!(self.2.to_string()))
    }
}

impl ServerError {
    pub fn not_found(ctx: middleware::WorkerContext, msg: &str) -> Self {
        Self(ctx, StatusCode::NOT_FOUND, anyhow::anyhow!(msg.to_string()))
    }
    pub fn bad_request(ctx: middleware::WorkerContext, msg: &str) -> Self {
        Self(
            ctx,
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!(msg.to_string()),
        )
    }
    pub fn unauthorized(ctx: middleware::WorkerContext, msg: &str) -> Self {
        Self(
            ctx,
            StatusCode::UNAUTHORIZED,
            anyhow::anyhow!(msg.to_string()),
        )
    }
    pub fn internal_error(ctx: middleware::WorkerContext, msg: &str) -> Self {
        Self(
            ctx,
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!(msg.to_string()),
        )
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> AxumResponse {
        let mut resp = (self.1, self.2.to_string()).into_response();
        resp.headers_mut()
            .insert("x-request-id", self.0.req_id.parse().unwrap());
        resp.headers_mut()
            .insert("x-server-by", self.0.endpoint.parse().unwrap());
        let exts = resp.extensions_mut();
        exts.insert(self);
        resp
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(
            middleware::WorkerContext {
                endpoint: ENDPOINT_NAME.get().unwrap().to_string(),
                ..Default::default()
            },
            StatusCode::INTERNAL_SERVER_ERROR,
            err.into(),
        )
    }
}
