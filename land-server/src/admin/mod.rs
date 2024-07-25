use crate::{
    dash::ServerError,
    templates::{new_handlebar, Engine},
};
use anyhow::Result;
use axum::{
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use axum_template::RenderHtml;
use land_vars::{AuthUser, BreadCrumbKey, Page};
use serde::Serialize;
use tower_http::services::ServeDir;

mod middle;
mod projects;
mod settings;
mod storage;
mod workers;

async fn handler(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
    }
    Ok(RenderHtml(
        "admin/index.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin", BreadCrumbKey::Admin, Some(user)),
        },
    ))
}

pub async fn route(assets_dir: &str, tpl_dir: Option<String>) -> Result<Router> {
    // Extract templates
    let hbs = new_handlebar(assets_dir, tpl_dir.clone())?;
    // set static assets directory
    let static_assets_dir = format!("{}/static", tpl_dir.unwrap_or(assets_dir.to_string()));

    let app = Router::new()
        .route("/", get(handler))
        .route("/projects", get(projects::index))
        .route("/settings", get(settings::index))
        .route("/settings/domains", post(settings::update_domains))
        .route("/settings/prometheus", post(settings::update_prometheus))
        .route("/storage", get(storage::index).post(storage::update))
        .route("/workers", get(workers::index))
        .route("/workers/tokens/create", post(workers::create_token))
        .route("/workers/tokens/remove", post(workers::remove_token))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .route_layer(middleware::from_fn(middle::check_admin))
        .route_layer(middleware::from_fn(crate::dash::middle::logger))
        .route_layer(middleware::from_fn(crate::dash::middle::auth))
        .with_state(Engine::from(hbs));
    Ok(app)
}
