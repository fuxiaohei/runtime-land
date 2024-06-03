use axum::body::Body;
use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::StatusCode;

/// response_redirect returns a redirect response
pub fn response_redirect(url: &str) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .unwrap()
}

pub struct ServerJsonError(pub StatusCode, pub anyhow::Error);

impl Clone for ServerJsonError {
    fn clone(&self) -> Self {
        Self(self.0, anyhow::anyhow!(self.1.to_string()))
    }
}

impl ServerJsonError {
    pub fn status_code(code: StatusCode, msg: &str) -> Self {
        Self(code, anyhow::anyhow!(msg.to_string()))
    }
}

#[derive(serde::Serialize)]
pub struct ServerJsonResponse {
    pub message: String,
}

impl IntoResponse for ServerJsonError {
    fn into_response(self) -> Response {
        let mut resp = (
            self.0,
            Json(ServerJsonResponse {
                message: self.1.to_string(),
            }),
        )
            .into_response();
        let exts = resp.extensions_mut();
        exts.insert(self);
        resp
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ServerJsonError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}

// Make our own error that wraps `anyhow::Error`.
pub struct ServerError(pub StatusCode, pub anyhow::Error);

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
