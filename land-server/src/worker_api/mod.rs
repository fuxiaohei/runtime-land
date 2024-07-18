use anyhow::Result;
use axum::{
    body::Body,
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

mod middle;
mod sync;

async fn handler() -> impl IntoResponse {
    Html("Hello World - Worker API !")
}

pub async fn route() -> Result<Router> {
    let app = Router::new()
        .route("/", get(handler))
        .route("/sync", post(sync::handle))
        .route_layer(middleware::from_fn(middle::auth));
    Ok(app)
}

#[derive(Debug, Serialize, Deserialize)]
struct CommonResponse<T> {
    pub status: String,
    pub message: String,
    pub data: T,
}

/// response_ok returns a response with status ok
fn response_ok(data: impl Serialize, msg: Option<String>) -> impl IntoResponse {
    let msg = msg.unwrap_or("ok".to_string());
    Json(CommonResponse {
        status: "ok".to_string(),
        message: msg,
        data,
    })
}

/// response_error returns a response with status error
fn response_error(msg: String) -> impl IntoResponse {
    Json(CommonResponse {
        status: "error".to_string(),
        message: msg,
        data: (),
    })
}

/// response_failed returns a response with error message
fn response_failed(status: StatusCode, msg: &str) -> impl IntoResponse {
    Response::builder()
        .status(status)
        .body(Body::from(msg.to_string()))
        .unwrap()
}

// Make our own error that wraps `anyhow::Error`.
pub struct JsonError(pub StatusCode, pub anyhow::Error);

impl Clone for JsonError {
    fn clone(&self) -> Self {
        Self(self.0, anyhow::anyhow!(self.1.to_string()))
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, RespError>`. That way you don't need to do that manually.
impl<E> From<E> for JsonError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}

// Tell axum how to convert `RespError` into a response.
impl IntoResponse for JsonError {
    fn into_response(self) -> axum::response::Response {
        let mut resp = response_error(self.1.to_string()).into_response();
        *resp.status_mut() = self.0;
        let exts = resp.extensions_mut();
        exts.insert(self);
        resp
    }
}
