use anyhow::Result;
use axum::Router;
use axum::{response::IntoResponse, routing::get};
use axum_template::engine::Engine;
use axum_template::RenderHtml;
use handlebars::{handlebars_helper, Handlebars};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{debug, info};
use walkdir::WalkDir;

mod embed;
pub use embed::extract_assets;

// RenderEngine is the template engine for axum_template
pub type RenderEngine = Engine<Handlebars<'static>>;

// basic handler that responds with a static string
async fn root(engine: RenderEngine) -> impl IntoResponse {
    RenderHtml("index.hbs", engine, &{})
}

/// router returns api server router
pub fn router(assets_dir: &str) -> Result<Router> {
    let static_assets_dir = format!("{}/static", assets_dir);
    let hbs = init_templates(assets_dir)?;
    let rt = Router::new()
        .route("/projects", get(root))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .with_state(Engine::from(hbs));
    Ok(rt)
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
