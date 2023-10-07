use super::params::{TemplateInfo, TemplateInfosMap};
use crate::apiv2::RouteError;
use axum::Json;
use hyper::StatusCode;
use land_core::metadata::Metadata;
use std::collections::HashMap;

#[tracing::instrument(name = "[templates_list_handler]", skip_all)]
pub async fn list_handler() -> Result<(StatusCode, Json<TemplateInfosMap>), RouteError> {
    let mut templates = HashMap::new();
    crate::embed::TemplateAssets::iter().for_each(|asset| {
        let asset_path = asset.to_string();
        let path = std::path::Path::new(&asset_path);

        // if path filename is land.toml, parse it
        if path.file_name().unwrap().to_str().unwrap() != "land.toml" {
            return;
        }

        let content = crate::embed::TemplateAssets::get(&asset_path).unwrap().data;
        let metadata = Metadata::from_binary(&content).unwrap();
        let language = metadata.language.clone();
        let template_info = build_template_info(&metadata);
        if let std::collections::hash_map::Entry::Vacant(e) = templates.entry(language.clone()) {
            e.insert(vec![template_info]);
        } else {
            let template_infos: &mut Vec<TemplateInfo> = templates.get_mut(&language).unwrap();
            template_infos.push(template_info);
        }
    });

    Ok((StatusCode::OK, Json(templates)))
}

fn build_template_info(metadata: &Metadata) -> TemplateInfo {
    let src_index_name = match metadata.language.as_str() {
        "rust" => "src/lib.rs",
        "javascript" => "src/index.js",
        _ => "",
    };
    let src_file = format!("{}/{}", metadata.name, src_index_name);
    let content = crate::embed::TemplateAssets::get(&src_file).unwrap().data;
    TemplateInfo {
        name: metadata.name.clone(),
        template_name: metadata.template_name.clone().unwrap_or("".to_string()),
        description: metadata.description.clone(),
        content: std::str::from_utf8(&content).unwrap().to_string(),
        language: metadata.language.clone(),
    }
}
