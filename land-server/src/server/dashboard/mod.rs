use super::templates::{RenderHtmlMinified, TemplateEngine};
use crate::server::PageVars;
use anyhow::Result;
use axum::{
    middleware,
    response::IntoResponse,
    routing::{any, get},
    Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use axum_template::engine::Engine;
use tower_http::services::ServeDir;

mod auth;
mod projects;

/// index is a handler for GET /
pub async fn index(engine: TemplateEngine) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
    }
    // redirect to /overview
    RenderHtmlMinified(
        "index.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Dashboard", "overview"),
        },
    )
}

/// router returns the router for the dashboard
pub fn router(assets_dir: &str) -> Result<Router> {
    super::templates::extract(assets_dir)?;

    // init handlebars template engine
    let hbs = super::templates::init(assets_dir)?;

    // set static assets directory
    let static_assets_dir = format!("{}/static", assets_dir);

    // set csrf config
    let config = CsrfConfig::default();

    let app = Router::new()
        .route("/", any(index))
        .route("/sign-in", get(auth::sign_in))
        .route("/sign-callback", get(auth::sign_callback))
        .route("/sign-out", get(auth::sign_out))
        .route("/projects", get(projects::index))
        .route("/new", get(projects::new))
        .route("/new/playground/:template", get(projects::new_playground))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .layer(CsrfLayer::new(config))
        .with_state(Engine::from(hbs))
        .route_layer(middleware::from_fn(auth::middleware));
    Ok(app)
}
