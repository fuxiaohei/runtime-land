use axum::{
    body::Body,
    http::{Request, Response},
    response::IntoResponse,
    routing::any,
    Router,
};
use hyper::StatusCode;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;

mod auth;
mod endpoint;
mod rest;

pub fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    Router::new()
        .merge(auth::router())
        .merge(endpoint::router())
        .merge(rest::router())
        .route("/v2/*path", any(v2_handler))
        .layer(cors)
}

async fn v2_handler(_req: Request<Body>) -> Response<Body> {
    let mut builder = Response::builder().status(200);
    builder = builder.header("x-land-version", land_core::version::get());
    builder
        .body(Body::from("Hello, Runtime.land! API v2"))
        .unwrap()
}

// Make our own error that wraps `anyhow::Error`.
pub struct RouteError(anyhow::Error, StatusCode);

#[derive(serde::Serialize)]
struct RouteErrorJson {
    pub message: String,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        error!("app error: {:?}", self.0);
        (
            self.1,
            axum::Json::from(RouteErrorJson {
                message: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, RouteErrorJson>`. That way you don't need to do that manually.
impl<E> From<E> for RouteError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
