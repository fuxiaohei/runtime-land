use crate::templates::{new_handlebar, Engine};
use anyhow::Result;
use axum::{response::IntoResponse, routing::get, Router};
use axum_template::RenderHtml;
use tower_http::services::ServeDir;

async fn handler(engine: Engine) -> impl IntoResponse {
    RenderHtml("index.hbs", engine, &())
}

pub async fn route(assets_dir: &str, tpl_dir: Option<String>) -> Result<Router> {
    // Extract templates
    let hbs = new_handlebar(assets_dir, tpl_dir.clone())?;
    // set static assets directory
    let static_assets_dir = format!("{}/static", tpl_dir.unwrap_or(assets_dir.to_string()));

    let app = Router::new()
        .route("/", get(handler))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .with_state(Engine::from(hbs));
    Ok(app)
}
