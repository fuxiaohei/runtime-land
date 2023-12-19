use anyhow::Result;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

mod cli;

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

/// router returns api server router
pub fn router() -> Router {
    let router = Router::new()
        .route("/", get(root))
        .route("/cli/login/*token", post(cli::login));
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
struct AppError(StatusCode, anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
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
