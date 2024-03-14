use anyhow::Result;
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use land_common::version;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, instrument, warn};

mod dash;
mod tpls;
mod worker;

/// start starts the server
#[instrument("[SVR]", skip_all)]
pub async fn start(assets_dir: &str, addr: SocketAddr) -> Result<()> {
    let dash_app = dash::router(assets_dir)?;
    let app = Router::new()
        .merge(dash_app)
        .nest("/api/worker/v1", worker::router()?)
        .route_layer(middleware::from_fn(log_middleware));
    info!(addr = addr.to_string(), "Start server");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// Make our own error that wraps `anyhow::Error`.
pub struct ServerError(StatusCode, anyhow::Error);

impl Clone for ServerError {
    fn clone(&self) -> Self {
        Self(self.0, anyhow::anyhow!(self.1.to_string()))
    }
}

impl ServerError {
    pub fn not_found(msg: &str) -> Self {
        Self(StatusCode::NOT_FOUND, anyhow::anyhow!(msg.to_string()))
    }
    pub fn bad_request(msg: &str) -> Self {
        Self(StatusCode::BAD_REQUEST, anyhow::anyhow!(msg.to_string()))
    }
    pub fn unauthorized(msg: &str) -> Self {
        Self(StatusCode::UNAUTHORIZED, anyhow::anyhow!(msg.to_string()))
    }
    pub fn internal_error(msg: &str) -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!(msg.to_string()),
        )
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let mut resp = (self.0, self.1.to_string()).into_response();
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
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}

/// PageVars is the common variables for all pages
#[derive(Debug, Default, Serialize)]
pub struct PageVars {
    pub title: String,
    pub base_uri: String,
    pub version: String,
    pub build_time: String,
    pub nav: String,
}

impl PageVars {
    pub fn new(title: &str, base_uri: &str, nav: &str) -> Self {
        Self {
            title: title.to_string(),
            base_uri: base_uri.to_string(),
            version: version::SHORT.to_string(),
            build_time: version::date(),
            nav: nav.to_string(),
        }
    }
}

/// redirect_response returns a redirect response
pub fn redirect_response(url: &str) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .unwrap()
}

/// not_modified_response returns a not modified response
pub fn not_modified_response() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .body(Body::empty())
        .unwrap()
}

/// log_middleware is a middleware for logging
#[instrument("[HTTP]", skip_all)]
async fn log_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();
    if path.starts_with("/static") {
        // ignore static assets log
        return Ok(next.run(request).await);
    }
    if path.starts_with("/api/worker/v1/deploys"){
        // high sequence url
        return Ok(next.run(request).await);
    }
    let method = request.method().clone().to_string();
    let st = tokio::time::Instant::now();
    let resp = next.run(request).await;
    let server_err = resp.extensions().get::<ServerError>();
    let status = resp.status().as_u16();
    let elasped = st.elapsed().as_millis();
    if let Some(err) = server_err {
        warn!(
            method = method,
            path = path,
            status = status,
            elasped = elasped,
            "Failed: {}",
            err.1
        );
    } else {
        let empty_header = HeaderValue::from_str("").unwrap();
        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap();
        let content_size = resp
            .headers()
            .get("content-length")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap();
        info!(
            method = method,
            path = path,
            status = status,
            cost = elasped,
            typ = content_type,
            size = content_size,
            "Ok",
        );
    }
    Ok(resp)
}
