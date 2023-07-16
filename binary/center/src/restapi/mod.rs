use axum::extract::DefaultBodyLimit;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware,
    routing::{any, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod params;

fn auth_router() -> Router {
    Router::new()
        .route("/v1/user/login", post(auth::login_by_email))
        .route("/v1/user/sign-up", post(auth::signup_email))
        .route("/v1/user/verify", post(auth::verify_token))
}

fn api_router() -> Router {
    Router::new()
        .route("/v1/home", get(default_handler))
        .route_layer(middleware::from_fn(auth::middleware))
}

/// default_handler is the default handler for all requests.
async fn default_handler(_req: Request<Body>) -> Response<Body> {
    let builder = Response::builder().status(200);
    builder.body(Body::from("Hello, land-center")).unwrap()
}

pub fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    Router::new()
        .merge(auth_router())
        .merge(api_router())
        .route("/", any(default_handler))
        .route("/*path", any(default_handler))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(cors)
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
            axum::Json::from(AppErrorJson {
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
