use anyhow::Result;
use land_dao::projects::Language;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "../examples"]
pub struct ExampleAssets;

/// TemplateVar is a struct to define playground template and content src
#[derive(Clone)]
pub struct TemplateVar {
    pub description: String,
    pub language: Language,
    pub content: String,
    pub src_file: String,
}

impl TemplateVar {
    pub fn from(name: &str) -> Result<Option<TemplateVar>> {
        let mut opt = PLAGROUND_TEMPLATES.get(name).cloned();
        if opt.is_some() {
            let var = opt.as_mut().unwrap();
            let content = ExampleAssets::get(&var.src_file).unwrap();
            var.content = String::from_utf8(content.data.to_vec()).unwrap();
        }
        Ok(opt)
    }
}

// a global map to define playground template and content src in ExampleAssets
static PLAGROUND_TEMPLATES: Lazy<HashMap<String, TemplateVar>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "js-hello".to_string(),
        TemplateVar {
            description: "a simple HTTP router that shows Hello World written in JavaScript"
                .to_string(),
            language: Language::JavaScript,
            content: String::new(),
            src_file: "js-hello/src/index.js".to_string(),
        },
    );
    map
});
