use anyhow::Result;
use axum::extract::Path;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::{
    body::Body,
    routing::{any, get, post},
    Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use axum_template::engine::Engine;
use handlebars::{handlebars_helper, Handlebars};
use hyper::StatusCode;
use mime_guess::mime;
use tower_http::services::ServeDir;
use tracing::debug;

mod account;
mod admin;
mod admin_deploy_tokens;
mod admin_deployments;
mod admin_projects;
mod admin_settings;
mod auth;
mod projects;
mod vars;

pub type AppEngine = Engine<Handlebars<'static>>;

pub fn router() -> Router {
    let hbs = init_templates().unwrap();
    let config = CsrfConfig::default();
    let mut router = Router::new()
        .route("/projects", get(projects::render))
        .route("/projects/:name", get(projects::render_single))
        .route("/projects/:name/settings", post(projects::handle_rename))
        .route("/projects/:name/settings", get(projects::render_settings))
        .route("/projects/:name/publish", get(projects::handle_publish))
        .route("/projects/:name/enable", get(projects::handle_enable))
        .route("/projects/:name/disable", get(projects::handle_disable))
        .route("/projects/:name/delete", post(projects::handle_delete))
        .route("/sign-in", get(auth::render_signin))
        .route("/sign-out", get(auth::render_signout))
        .route("/sign-callback/*path", get(auth::clerk_callback))
        .route("/account/settings", get(account::render_settings))
        .route(
            "/account/settings/create-token",
            post(account::handle_create_token),
        )
        .route(
            "/account/settings/delete-token",
            get(account::handle_delete_token),
        )
        .route(
            "/admin/projects",
            get(admin_projects::render).post(admin_projects::handle),
        )
        .route(
            "/admin/deployments",
            get(admin_deployments::render).post(admin_deployments::handle),
        )
        .route("/admin/users", get(admin::render_users))
        .route(
            "/admin/runtime-nodes",
            get(admin_settings::render_runtime_nodes),
        )
        .route(
            "/admin/storage",
            get(admin_settings::render_storage).post(admin_settings::handle_storage),
        )
        .route(
            "/admin/domains",
            get(admin_settings::render_domains).post(admin_settings::handle_domains),
        )
        .route("/admin/deploy-tokens", get(admin_deploy_tokens::render))
        .route("/*path", any(render_notfound));
    if cfg!(debug_assertions) {
        router = router.route("/static/*path", get(render_static));
    } else {
        router = router.nest_service("/static", ServeDir::new("static"))
    }

    router
        .layer(CsrfLayer::new(config))
        .with_state(Engine::from(hbs))
        .route_layer(middleware::from_fn(auth::session_auth_middleware))
}

async fn render_notfound() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("page not found"))
        .unwrap()
}

async fn render_static(Path(path): Path<String>) -> Response<Body> {
    // if path not exist in TemplateAssets, return 404
    let content = crate::embed::TemplatesAssets::get(&path);
    if content.is_none() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("page not found"))
            .unwrap();
    }
    let guess = mime_guess::from_path(&path);
    let content_type = guess.first().unwrap_or(mime::TEXT_PLAIN);
    Response::builder()
        .header("content-type", content_type.to_string())
        .status(StatusCode::OK)
        .body(Body::from(content.unwrap().data))
        .unwrap()
}

// add handlebars_helper to handle if value is equal args, return "selected" for Option element
handlebars_helper!(selected: |x: str, y: str| if x == y { "selected" } else { "" });

fn init_templates() -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();
    hbs.register_helper("selected", Box::new(selected));
    load_template_from_assets(&mut hbs);
    Ok(hbs)
}

fn load_template_from_assets(hbs: &mut Handlebars<'static>) {
    crate::embed::TemplatesAssets::iter().for_each(|asset| {
        let asset_path = asset.to_string();

        // if asset_path is suffixed with .hbs or .html
        if !asset_path.ends_with(".hbs") && !asset_path.ends_with(".html") {
            return;
        }

        let content = crate::embed::TemplatesAssets::get(&asset_path)
            .unwrap()
            .data;

        debug!("register template: {}", asset_path);
        hbs.register_template_string(&asset_path, std::str::from_utf8(&content).unwrap())
            .unwrap();
    });
}
