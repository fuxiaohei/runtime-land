use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

async fn handler() -> impl IntoResponse {
    Html("Hello World - Worker API !")
}

pub async fn route() -> Result<Router> {
    let app = Router::new().route("/", get(handler));
    Ok(app)
}
