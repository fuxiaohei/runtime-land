use super::tpls;
use anyhow::Result;
use axum::{
    body::Body,
    http::{header::LOCATION, Response, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{any, get, post},
    Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use axum_template::engine::Engine;
use tower_http::services::ServeDir;

mod auth;
mod overview;
mod playground;
mod projects;
mod settings;

/// index is a handler for GET /
pub async fn index() -> impl IntoResponse {
    // redirect to /overview
    Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header(LOCATION, "/overview")
        .body(Body::empty())
        .unwrap()
}

/// router returns the router for the dashboard
pub fn router(assets_dir: &str) -> Result<Router> {
    // extract all assets to the static directory
    tpls::extract(assets_dir)?;

    // init handlebars template engine
    let hbs = tpls::init(assets_dir)?;

    // set csrf config
    let config = CsrfConfig::default();

    // set static assets directory
    let static_assets_dir = format!("{}/static", assets_dir);

    let app = Router::new()
        .route("/", any(index))
        .route("/sign-in", get(auth::sign_in))
        .route("/sign-callback", get(auth::sign_callback))
        .route("/sign-out", get(auth::sign_out))
        .route("/overview", get(overview::index))
        .route("/new", get(projects::new))
        .route("/new/blank", get(projects::new_blank))
        .route("/new/playground/:template", get(playground::new))
        .route("/projects", get(projects::index))
        .route("/projects/:name", get(projects::single))
        .route("/projects/:name/traffic", get(projects::traffic))
        .route(
            "/projects/:name/settings",
            get(projects::settings).post(projects::settings_post_domain),
        )
        .route("/projects/:name/delete", post(projects::post_delete))
        .route(
            "/playground/:name",
            get(playground::index).post(playground::save),
        )
        .route("/playground/:name/check", get(playground::check))
        .route("/settings", get(settings::index))
        .route("/settings/create-token", post(settings::create_token))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .layer(CsrfLayer::new(config))
        .with_state(Engine::from(hbs))
        .route_layer(middleware::from_fn(auth::middleware));
    Ok(app)
}
