use anyhow::Result;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::{
    body::Body,
    routing::{any, get},
    Router,
};
use axum_template::engine::Engine;
use axum_template::RenderHtml;
use handlebars::Handlebars;
use hyper::StatusCode;
use mime_guess::mime;
use tracing::debug;

// Type alias for our engine. For this example, we are using Handlebars
type AppEngine = Engine<Handlebars<'static>>;

pub fn router() -> Router {
    let hbs = init_templates().unwrap();
    Router::new()
        .route("/demo", any(render_demo))
        .route("/static/*path", get(render_static))
        .with_state(Engine::from(hbs))
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

async fn render_demo(
    // Obtain the engine
    engine: AppEngine,
) -> impl IntoResponse {
    RenderHtml("demo", engine, &())
}

fn init_templates() -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();

    crate::embed::TemplatesAssets::iter().for_each(|asset| {
        let asset_path = asset.to_string();

        // if asset_path is suffixed with .hbs or .html
        if !asset_path.ends_with(".hbs") && !asset_path.ends_with(".html") {
            return;
        }

        let content = crate::embed::TemplatesAssets::get(&asset_path)
            .unwrap()
            .data;

        // template name drop suffix .html and .hbs
        let template_name = asset_path.replace(".html", "").replace(".hbs", "");
        debug!("register template: {}", template_name);
        hbs.register_template_string(&template_name, std::str::from_utf8(&content).unwrap())
            .unwrap();
    });
    Ok(hbs)
}
