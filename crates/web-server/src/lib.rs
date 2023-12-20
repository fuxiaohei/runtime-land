use anyhow::Result;
use axum::extract::MatchedPath;
use axum::http::{Request, StatusCode};
use axum::response::{Redirect, Response};
use axum::routing::{any, post};
use axum::{middleware, Router};
use axum::{response::IntoResponse, routing::get};
use axum_csrf::{CsrfConfig, CsrfLayer};
use axum_template::engine::Engine;
use axum_template::RenderHtml;
use handlebars::{handlebars_helper, Handlebars};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, info_span, warn, Span};
use walkdir::WalkDir;

mod embed;
pub use embed::extract_assets;

mod admin;
mod projects;
mod settings;
mod sign;

// RenderEngine is the template engine for axum_template
pub type RenderEngine = Engine<Handlebars<'static>>;

/// router returns api server router
pub fn router(assets_dir: &str) -> Result<Router> {
    let static_assets_dir = format!("{}/static", assets_dir);
    let hbs = init_templates(assets_dir)?;
    let config = CsrfConfig::default();

    let admin_rt = Router::new()
        .route("/dashboard", get(admin::dashboard))
        .route("/storage", get(admin::storage));

    let rt = Router::new()
        .route("/sign-in", get(sign::signin))
        .route("/sign-callback/*path", get(sign::signcallback))
        .route("/projects", get(projects::index))
        .route("/settings", get(settings::index))
        .route(
            "/settings/token",
            post(settings::create_token).delete(settings::delete_token),
        )
        .nest("/admin", admin_rt)
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .route("/*path", any(not_found))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    let uri = request.uri().to_string();

                    info_span!(
                        "http",
                        method = ?request.method(),
                        uri = %uri,
                        matched_path,
                        cost = tracing::field::Empty,
                        status = tracing::field::Empty,
                    )
                })
                .on_response(|response: &Response, latency: Duration, span: &Span| {
                    span.record("cost", latency.as_millis());
                    span.record("status", response.status().as_u16());
                    if response.status().is_success() {
                        info!("success")
                    } else {
                        warn!("failure")
                    }
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                        span.record("cost", latency.as_millis());
                        error!("error, {}", error)
                    },
                ),
        )
        .layer(CsrfLayer::new(config))
        .with_state(Engine::from(hbs))
        .route_layer(middleware::from_fn(sign::auth));
    Ok(rt)
}

/// default_handler is the default handler for all routes
pub async fn default_handler() -> impl IntoResponse {
    Redirect::permanent("/projects")
}

/// not_found is the handler for 404
async fn not_found(engine: RenderEngine) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
    }

    RenderHtml(
        "page_not_found.hbs",
        engine,
        Vars {
            page: PageVars::new("Page Not Found", ""),
        },
    )
}

/// run starts api server
pub async fn run(addr: SocketAddr, assets_dir: &str) -> Result<()> {
    let app = router(assets_dir)?;

    info!("Starting on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

// add handlebars_helper to handle if value is equal args, return "selected" for Option element
handlebars_helper!(selected: |x: str, y: str| if x == y { "selected" } else { "" });

fn init_templates(dir: &str) -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();
    hbs.register_helper("selected", Box::new(selected));

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let extension = path.extension().unwrap_or_default();
            if extension != "hbs" && extension != "html" {
                continue;
            }
            // get relative path from dir
            let content = std::fs::read_to_string(path)?;
            let tpl_name = path.strip_prefix(dir).unwrap().to_str().unwrap();
            debug!("register template: {:?}", tpl_name);
            hbs.register_template_string(tpl_name, content)?;
        }
    }
    Ok(hbs)
}

/// PageVars is the common variables for all pages
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageVars {
    pub title: String,
    pub base_uri: String,
    pub version: String,
    pub build_time: String,
}

impl PageVars {
    pub fn new(title: &str, base_uri: &str) -> Self {
        Self {
            title: title.to_string(),
            base_uri: base_uri.to_string(),
            version: land_common::build_info(),
            build_time: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(StatusCode, anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1.to_string()).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}
