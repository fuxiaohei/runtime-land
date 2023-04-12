use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../examples"]
pub struct TemplateAssets;