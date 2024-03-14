use anyhow::{anyhow, Result};
use clap::Args;
use color_print::cprintln;
use inquire::{validator::Validation, CustomUserError, Select, Text};
use land_common::manifest::MANIFEST_FILE;
use tracing::debug;

use crate::embed::ExampleAssets;

/// Command New
#[derive(Args, Debug)]
pub struct New {
    /// The name of the new project
    #[clap(value_parser = validate_name)]
    pub name: Option<String>,

    /// The template from which to create the new project
    #[clap(short = 't', long = "template")]
    pub template: Option<String>,

    /// The description of the new project
    #[clap(short = 'd', long = "desc")]
    pub desc: Option<String>,
}

impl New {
    pub async fn run(&self) -> Result<()> {
        debug!("Create new project: {:?}", self);

        let project_name = get_project_name(self.name.clone())?;
        let template_name = get_template_name(self.template.clone())?;
        let desc = get_desc(self.desc.clone())?;

        cprintln!("<green>Create project '{}'</green>", project_name);

        // check if dir not exist
        if !std::path::Path::new(&project_name).exists() {
            std::fs::create_dir(&project_name)?;
            cprintln!("Create directory '{}'", project_name)
        }

        // extract template into project dir
        extract_template(template_name, &project_name, &desc).await?;
        Ok(())
    }
}

/// validate_name validates the name of the project
fn validate_name(name: &str) -> Result<String, String> {
    // name only support alphabet, number and "-", and start with alphabet
    let re = regex::Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
    if !re.is_match(name) {
        return Err("Project name only support alphabet, number and \"-\"".to_string());
    }
    if name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
        return Err("Project name must start with alphabet".to_string());
    }
    Ok(name.to_string())
}

fn get_project_name(name: Option<String>) -> Result<String> {
    if let Some(name) = name {
        return Ok(name);
    }
    let name_validator = |input: &str| match validate_name(input) {
        Ok(_name) => Ok::<Validation, CustomUserError>(Validation::Valid),
        Err(err) => Ok(Validation::Invalid(err.to_string().into())),
    };
    match Text::new("Enter a name for your project:")
        .with_validator(name_validator)
        .prompt()
    {
        Ok(name) => Ok(name),
        Err(_err) => Err(anyhow!(
            "Project name only support alphabet, number and \"-\", and start with alphabet"
        )),
    }
}

#[derive(Debug)]
struct TemplateMeta {
    pub name: String,
    pub desc: String,
}

impl std::fmt::Display for TemplateMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.desc)
    }
}

fn default_templates() -> Vec<TemplateMeta> {
    vec![
        TemplateMeta {
            name: "http-rust".to_string(),
            desc: "HTTP request handler using Rust".to_string(),
        },
        TemplateMeta {
            name: "http-js".to_string(),
            desc: "HTTP request handler using JavaScript".to_string(),
        },
    ]
}

fn get_template_name(name: Option<String>) -> Result<TemplateMeta> {
    if let Some(name) = name {
        return Ok(TemplateMeta {
            name,
            desc: String::new(),
        });
    }
    let templates = default_templates();
    match Select::new("Pick a template to start your project:", templates).prompt() {
        Ok(template) => Ok(template),
        Err(_err) => Err(anyhow!("Template not found")),
    }
}

fn get_desc(desc: Option<String>) -> Result<String> {
    if let Some(desc) = desc {
        return Ok(desc);
    }
    match Text::new("Enter a description for your project (Optional):").prompt() {
        Ok(desc) => Ok(desc),
        Err(_err) => Err(anyhow!("Project description is invalid")),
    }
}

async fn extract_template(template: TemplateMeta, dir: &str, mut desc: &str) -> Result<()> {
    let template_name = &template.name;
    if desc.is_empty() {
        desc = &template.desc;
    }
    // metadata file {tpl_name}/land.toml should exist in embed asserts
    let metadata_file = format!("{}/land.toml", template_name);
    let file = ExampleAssets::get(&metadata_file);
    if file.is_none() {
        return Err(anyhow!("Template '{}' not found", template_name));
    }
    // extract template
    for item in ExampleAssets::iter() {
        if !item.starts_with(template_name) {
            continue;
        }
        // calculate target file and content from a template file
        let (target_file, content) = cal_template_content(item.as_ref(), dir, template_name, desc);

        // create parent dir of target file
        let parent_dir = std::path::Path::new(&target_file).parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
            cprintln!("Extract directory {:?}", parent_dir);
        }

        std::fs::write(&target_file, content)?;
        cprintln!("Extract file {:?}", target_file);
    }
    Ok(())
}

fn cal_template_content(
    item: &str,
    dir: &str,
    template_name: &str,
    desc: &str,
) -> (String, Vec<u8>) {
    let target_file = format!("{}{}", dir, item.replace(template_name, ""));
    let raw_file = ExampleAssets::get(item).unwrap();
    // if item is manifest file, refresh it project name and description
    if item.ends_with(MANIFEST_FILE) {
        let mut data = String::from_utf8(raw_file.data.to_vec()).unwrap();
        data = data.replace("{{project_name}}", dir);
        data = data.replace("{{description}}", desc);
        data = data.replace("{{project_wasm}}", dir.replace('-', "_").as_str());
        return (target_file, data.as_bytes().to_vec());
    }
    // if item is Cargo.toml.txt, refresh it project name and sdk version
    if item.ends_with("Cargo.toml.txt") {
        let sdk_version =
            std::env::var("SDK_VERSION").unwrap_or(format!("\"{}\"", env!("CARGO_PKG_VERSION")));
        let mut data = String::from_utf8(raw_file.data.to_vec()).unwrap();
        data = data.replace("{{project_name}}", dir);
        data = data.replace("{{sdk_version}}", &sdk_version);
        data = data.replace("{{description}}", desc);
        return (target_file.replace(".txt", ""), data.as_bytes().to_vec());
    }
    (target_file, raw_file.data.to_vec())
}
