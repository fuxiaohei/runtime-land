use super::{
    templates::{RenderHtmlMinified, TemplateEngine},
    ServerError,
};
use crate::server::{
    dashboard::{auth::SessionUser, vars::ProjectVar},
    PageVars,
};
use anyhow::Result;
use axum::{
    middleware,
    response::IntoResponse,
    routing::{any, get},
    Extension, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use axum_template::engine::Engine;
use tower_http::services::ServeDir;
use tracing::info;

mod auth;
mod projects;
mod vars;

/// index is a handler for GET /
pub async fn index(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        projects: Vec<ProjectVar>,
    }

    // list recent updated projects
    // the overview page show 5 cards of the recent updated projects
    let projects_data = land_dao::projects::list_by_user_id(user.id, None, 5).await?;
    info!("Overview projects: {}", projects_data.len());
    let projects = ProjectVar::from_models_vec(projects_data).await?;

    Ok(RenderHtmlMinified(
        "index.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Dashboard", "overview"),
            user,
            projects,
        },
    ))
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
        .route("/projects/:name", get(projects::one))
        .route("/playground/:name", get(projects::show_playground))
        .route("/new", get(projects::new))
        .route("/new/playground/:template", get(projects::new_playground))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .layer(CsrfLayer::new(config))
        .with_state(Engine::from(hbs))
        .route_layer(middleware::from_fn(auth::middleware));
    Ok(app)
}
