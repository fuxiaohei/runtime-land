use anyhow::Result;
use axum::extract::{MatchedPath, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, warn, Span};

mod cli;

/// router returns api server router
pub fn router() -> Router {
    let router = Router::new()
        .route("/cli/login/*token", post(cli::login))
        .route("/cli/deploy", post(cli::deploy))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    let uri = request.uri().to_string();

                    info_span!(
                        "api/v2",
                        method = ?request.method(),
                        uri = %uri,
                        matched_path,
                        cost = tracing::field::Empty,
                        status = tracing::field::Empty,
                    )
                })
                .on_response(|response: &Response, latency: Duration, span: &Span| {
                    span.record("cost", latency.as_millis());
                    span.record("status", response.status().as_u16());
                    if response.status().is_success() {
                        info!("success")
                    } else if response.status().is_server_error()
                        || response.status().is_client_error()
                    {
                        warn!("failure, status: {:?}", response.status(),)
                    } else {
                        info!("30x")
                    }
                }),
        );
    Router::new().nest("/api/v2", router)
}

/// run starts api server
pub async fn run(addr: SocketAddr) -> Result<()> {
    let app = router();

    info!("Starting on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
struct AppError(StatusCode, anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("AppError: {:?}", self);
        (self.0, self.1.to_string()).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}
