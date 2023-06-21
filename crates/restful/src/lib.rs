use anyhow::Result;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{Request, Response, StatusCode};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{any, delete, get, post};
use axum::{Json, Router};
use std::net::SocketAddr;
use tracing::info;

mod auth;
mod deployments;
mod login;
mod params;
mod projects;
mod tokens;

pub mod client;

// basic handler that responds with a static string
async fn default_handler(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from("Hello, Runtime.land!"))
}

fn login_router() -> Router {
    Router::new()
        .route("/v1/signup-email", post(login::signup_email))
        .route("/v1/login-by-email", post(login::login_by_email))
        .route("/v1/login-by-token", post(login::login_by_token))
}

fn api_router() -> Router {
    Router::new()
        .route("/v1/tokens", get(tokens::list_handler))
        .route("/v1/tokens", post(tokens::create_handler))
        .route("/v1/tokens", delete(tokens::remove_handler))
        .route("/v1/project", get(projects::fetch_handler))
        .route("/v1/project", post(projects::create_handler))
        .route("/v1/projects", get(projects::list_handler))
        .route("/v1/project/overview", get(projects::overview_handler))
        .route("/v1/deployment", post(deployments::create_handler))
        .route("/v1/deployment/publish", post(deployments::publish_handler))
        .route_layer(middleware::from_fn(auth::auth))
}

pub async fn start_server(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .merge(login_router())
        .merge(api_router())
        .route("/", any(default_handler))
        .route("/*path", any(default_handler))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error, StatusCode);

#[derive(serde::Serialize)]
struct AppErrorJson {
    pub message: String,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            self.1,
            Json::from(AppErrorJson {
                message: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
