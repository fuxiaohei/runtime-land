use anyhow::Result;
use axum::routing::post;
use axum::Router;

pub mod tokens;

pub fn router() -> Result<Router> {
    Ok(Router::new().route("/token", post(tokens::create)))
}
