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
        .route("/v1/token", post(tokens::create))
        .route("/v1/projects", get(projects::list))
        .route(
            "/v1/projects/:project_name",
            get(projects::single)
                .post(projects::update_names)
                .delete(projects::delete),
        )
        .route_layer(middleware::from_fn(clerk::middleware))
        .layer(cors))
}
