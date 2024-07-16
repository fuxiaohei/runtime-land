use crate::templates::{new_handlebar, Engine};
use anyhow::Result;
use axum::{response::IntoResponse, routing::get, Router};
use axum_template::RenderHtml;
use land_vars::{BreadCrumbKey, Page};
use serde::Serialize;
use tower_http::services::ServeDir;

mod auth;

async fn handler(engine: Engine) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
    }
    RenderHtml(
        "index.hbs",
        engine,
        Vars {
            page: Page::new("Dashboard", BreadCrumbKey::Home, None),
        },
    )
}

pub async fn route(assets_dir: &str, tpl_dir: Option<String>) -> Result<Router> {
    // Extract templates
    let hbs = new_handlebar(assets_dir, tpl_dir.clone())?;
    // set static assets directory
    let static_assets_dir = format!("{}/static", tpl_dir.unwrap_or(assets_dir.to_string()));

    let app = Router::new()
        .route("/", get(handler))
        .route("/sign-in",get(auth::sign_in))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .with_state(Engine::from(hbs));
    Ok(app)
}
