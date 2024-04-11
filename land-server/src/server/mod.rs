use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use http::HeaderValue;
use serde::Serialize;
use std::net::SocketAddr;
use tracing::{info, instrument, warn};

mod dashboard;
mod templates;

/// start the server
pub async fn start(addr: SocketAddr, assets_dir: &str) -> anyhow::Result<()> {
    let dashboard_router = dashboard::router(assets_dir)?;
    let app = Router::new()
        .merge(dashboard_router)
        .route_layer(middleware::from_fn(log_middleware));

    info!("Starting server on {}", addr);
    // run it
    let listener = tokio::net::TcpListener::bind(addr).await?;
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
    /// status_code creates a new `ServerError` with the given status code and message.
    pub fn status_code(code: StatusCode, msg: &str) -> Self {
        Self(code, anyhow::anyhow!(msg.to_string()))
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

/// redirect_response returns a redirect response
pub fn redirect_response(url: &str) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .unwrap()
}

/// PageVars is the common variables for all pages
#[derive(Debug, Default, Serialize)]
pub struct PageVars {
    pub title: String,
    pub version: String,
    pub build_time: String,
    pub nav: String,
}

impl PageVars {
    pub fn new(title: &str, nav: &str) -> Self {
        Self {
            title: title.to_string(),
            version: land_common::version::SHORT.to_string(),
            build_time: land_common::version::date(),
            nav: nav.to_string(),
        }
    }
}

#[instrument("[HTTP]", skip_all)]
async fn log_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();
    if path.starts_with("/static") {
        // ignore static assets log
        return Ok(next.run(request).await);
    }
    if path.starts_with("/api/worker/v1/deploys") {
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
