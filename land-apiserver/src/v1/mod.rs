use anyhow::Result;
use axum::routing::post;
use axum::Router;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

pub mod tokens;

pub fn router() -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    Ok(Router::new()
        .route("/token", post(tokens::create))
        .layer(cors))
}

#[derive(Serialize)]
pub struct Data<T: Serialize> {
    data: T,
}
