use anyhow::{anyhow, Result};
use clap::Args;
use color_print::cprintln;
use inquire::validator::Validation;
use inquire::{CustomUserError, Select, Text};
use land_core_service::metadata;
use std::path;
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
        let tpl = get_template_name(self.template.clone())?;
        let desc = get_desc(self.desc.clone())?;
        debug!(
            "Project name: {}, template: {}, desc: {}",
            project_name, tpl, desc
        );

        cprintln!("<green>Create project '{}'</green>", project_name);

        // check dir named 'project_name' exists
        if !path::Path::new(&project_name).exists() {
            // create dir named 'project_name'
            std::fs::create_dir(&project_name)?;
        }

        extract_tpl(&project_name, &tpl, &desc)?;

        Ok(())
    }
}

fn extract_tpl(dir: &str, tpl: &TemplateMeta, desc: &str) -> Result<()> {
    let metadata_file = format!("{}/{}", tpl.name, metadata::DEFAULT_FILE);
    let file = ExampleAssets::get(&metadata_file);
    if file.is_none() {
        return Err(anyhow!("Template '{}' not found", tpl.name));
    }
    // extract template
    for item in ExampleAssets::iter() {
        if !item.starts_with(&tpl.name) {
            continue;
        }
        let raw_file = ExampleAssets::get(&item).unwrap();
        let target_file = format!("{}{}", dir, item.replace(&tpl.name, ""));
        let target_dir = path::Path::new(&target_file).parent().unwrap();
        if !target_dir.exists() {
            std::fs::create_dir_all(target_dir)?;
        }
        std::fs::write(&target_file, raw_file.data)?;
        debug!("Extract file: {}", target_file);
    }
    // refresh toml file
    let meta_desc = if desc.is_empty() {
        tpl.desc.clone()
    } else {
        desc.to_string()
    };
    refresh_toml(dir, &meta_desc)?;
    Ok(())
}

fn refresh_toml(dir: &str, desc: &str) -> Result<()> {
    let toml_file = format!("{}/{}", dir, metadata::DEFAULT_FILE);
    let mut meta = metadata::Data::from_file(&toml_file)?;
    meta.name = dir.to_string();
    meta.description = desc.to_string();
    meta.to_file(&toml_file)?;
    Ok(())
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
    vec![TemplateMeta {
        name: "js-hello".to_string(),
        desc: "Simple hello world using Javascript".to_string(),
    }]
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
