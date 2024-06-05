use anyhow::Result;
use axum::routing::{get, post};
use axum::{middleware, Router};
use land_service::clerk;
use tower_http::cors::{Any, CorsLayer};

mod projects;
mod tokens;

pub fn router() -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    Ok(Router::new()
        .route("/token", post(tokens::create))
        .route("/projects", get(projects::list))
        .route_layer(middleware::from_fn(clerk::middleware))
        .layer(cors))
}
