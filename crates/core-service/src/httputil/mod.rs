use axum::extract::{ConnectInfo, Request};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use reqwest::StatusCode;
use std::net::SocketAddr;
use tracing::{info, instrument, warn};

mod response;
pub use response::*;

#[instrument("[HTTP]", skip_all)]
pub async fn log_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();
    if path.starts_with("/static") {
        // ignore static assets log
        return Ok(next.run(request).await);
    }
    let mut remote = "0.0.0.0".to_string();
    // if x-real-ip exists, use it
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        remote = real_ip.to_str().unwrap().to_string();
    } else if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        remote = connect_info.to_string();
    }

    if path.starts_with("/api/v1/worker-api/alive") {
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
            remote = remote,
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
            remote = remote,
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
