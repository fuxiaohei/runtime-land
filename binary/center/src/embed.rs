use rust_embed::RustEmbed;

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
