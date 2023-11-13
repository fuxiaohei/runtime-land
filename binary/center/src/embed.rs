use anyhow::Result;
use rust_embed::RustEmbed;
use tracing::debug;

#[derive(RustEmbed)]
#[folder = "../../examples"]
pub struct ExamplesAssets;

#[derive(RustEmbed)]
#[folder = "./templates"]
#[include = "*.hbs"]
#[include = "*.html"]
#[include = "*.css"]
#[include = "*.js"]
#[include = "*.png"]
pub struct TemplatesAssets;

/// extract_assets extracts all assets to the statis directory.
pub fn extract_assets(dir: &str) -> Result<()> {
    TemplatesAssets::iter().for_each(|file| {
        let filepath = file.to_string();
        
        // if filepath is suffixed with .hbs or .html, do not extract
        // other files extract to 'dir' directory with filepath directory
        if filepath.ends_with(".hbs") || filepath.ends_with(".html") {
            return;
        }
        let content = TemplatesAssets::get(&filepath).unwrap().data;
        let mut path = std::path::PathBuf::from(dir);
        path.push(filepath);
        debug!("extract asset: {:?}", path);

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    });
    Ok(())
}
