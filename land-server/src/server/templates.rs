use anyhow::Result;
use axum::response::IntoResponse;
use axum_template::engine::Engine;
use handlebars::{handlebars_helper, Handlebars};
use rust_embed::RustEmbed;
use tracing::debug;
use walkdir::WalkDir;

#[derive(RustEmbed)]
#[folder = "./templates"]
#[include = "*.hbs"]
#[include = "*.html"]
#[include = "*.css"]
#[include = "*.js"]
#[include = "*.png"]
#[include = "*.jpg"]
#[include = "*.ico"]
pub struct TemplateAssets;

/// TemplateEngine is the template engine for axum_template
pub type TemplateEngine = Engine<Handlebars<'static>>;

// add handlebars_helper to handle if value is equal args, return "selected" for Option element
handlebars_helper!(selected: |x: str, y: str| if x == y { "selected" } else { "" });
handlebars_helper!(is_nav_active: |x: str, y: str| if x == y { "active" } else { "" });
handlebars_helper!(is_js_project: |x: str| x.to_lowercase() == "js" || x.to_lowercase() == "javascript" );
handlebars_helper!(is_rust_project: |x: str|  x.to_lowercase() == "rust" );
handlebars_helper!(is_ts_project: |x: str|  x.to_lowercase() == "ts" || x.to_lowercase() == "typescript");
handlebars_helper!(eq_str: |x: str, y: str| x == y);

/// init initializes the handlebars template engine
pub fn init(dir: &str) -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();

    // register handlebars_helper
    hbs.register_helper("selected", Box::new(selected));
    hbs.register_helper("is_nav_active", Box::new(is_nav_active));
    hbs.register_helper("is_js_project", Box::new(is_js_project));
    hbs.register_helper("is_rust_project", Box::new(is_rust_project));
    hbs.register_helper("is_ts_project", Box::new(is_ts_project));
    hbs.register_helper("eq_str", Box::new(eq_str));

    // register templates
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let extension = path.extension().unwrap_or_default();
        if extension != "hbs" && extension != "html" {
            continue;
        }
        // get relative path from dir
        let content = std::fs::read_to_string(path)?;
        let tpl_name = path.strip_prefix(dir).unwrap().to_str().unwrap();
        debug!(name = tpl_name, "Register template");
        hbs.register_template_string(tpl_name, content)?;
    }
    Ok(hbs)
}

/// extract extracts all assets to the statis directory.
pub fn extract(dir: &str) -> Result<()> {
    TemplateAssets::iter().for_each(|file| {
        let filepath = file.to_string();

        let content = TemplateAssets::get(&filepath).unwrap().data;
        let mut path = std::path::PathBuf::from(dir);
        path.push(filepath);
        debug!(path = path.to_str(), "Extract asset");

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    });
    Ok(())
}

pub struct RenderHtmlMinified<K, E, S>(pub K, pub E, pub S);

impl<K, E, S> IntoResponse for RenderHtmlMinified<K, E, S>
where
    E: axum_template::TemplateEngine,
    S: serde::Serialize,
    K: AsRef<str>,
{
    fn into_response(self) -> axum::response::Response {
        let RenderHtmlMinified(key, engine, data) = self;

        let result = engine.render(key.as_ref(), data);
        match result {
            Ok(x) => {
                let mut cfg = minify_html::Cfg::spec_compliant();
                cfg.minify_js = true;
                cfg.minify_css = true;
                let minified = minify_html::minify(x.as_bytes(), &cfg);
                axum::response::Html(minified).into_response()
            }
            Err(x) => x.into_response(),
        }
    }
}
