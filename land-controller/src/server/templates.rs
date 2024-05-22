use anyhow::Result;
use rust_embed::RustEmbed;

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

/// extract extracts all assets to the statis directory.
pub fn extract(dir: &str) -> Result<()> {
    TemplateAssets::iter().for_each(|file| {
        let filepath = file.to_string();

        let content = TemplateAssets::get(&filepath).unwrap().data;
        let mut path = std::path::PathBuf::from(dir);
        path.push(filepath);
        // debug!(path = path.to_str(), "Extract asset");

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    });
    Ok(())
}
