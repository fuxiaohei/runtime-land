use clap::Args;
use color_print::cprintln;
use inquire::validator::Validation;
use inquire::{CustomUserError, Select, Text};
use regex::Regex;

use crate::embed::TemplateAssets;

/// Command Init
#[derive(Args, Debug)]
pub struct New {
    /// The name of the new project
    #[clap(value_parser = validate_name)]
    pub name: Option<String>,

    /// The template from which to create the new project
    #[clap(short = 't', long = "template", value_parser = validate_template)]
    pub template: Option<String>,
}

impl New {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let name = get_project_name(self.name.clone());
        let template = get_template_name(self.template.clone());
        let desc = match Text::new("Enter a description for your project (Optional):").prompt() {
            Ok(desc) => desc,
            Err(_err) => {
                // eprintln!("invalid project description: {}", err);
                std::process::exit(1);
            }
        };

        println!("name: {}, template: {}, desc: {}", name, template, desc);
        create_project(&name, &template, &desc)?;
        Ok(())
    }
}

fn get_project_name(name: Option<String>) -> String {
    if let Some(name) = name {
        return name;
    }
    let name_validator = |input: &str| match validate_name(input) {
        Ok(_name) => Ok::<Validation, CustomUserError>(Validation::Valid),
        Err(err) => Ok(Validation::Invalid(err.to_string().into())),
    };
    match Text::new("Enter a name for your project:")
        .with_validator(name_validator)
        .prompt()
    {
        Ok(name) => name,
        Err(_err) => {
            // eprintln!("invalid project name: {}", err);
            std::process::exit(1);
        }
    }
}

/// validate_name validates the name of the project
fn validate_name(name: &str) -> Result<String, String> {
    // name only support alphabet, number and "-", and start with alphabet
    let re = Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
    if !re.is_match(name) {
        return Err("project name only support alphabet, number and \"-\"".to_string());
    }
    if name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
        return Err("project name must start with alphabet".to_string());
    }
    Ok(name.to_string())
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

fn validate_template(name: &str) -> Result<String, String> {
    let templates = default_templates();
    for template in &templates {
        if template.name == name {
            return Ok(name.to_string());
        }
    }
    let names = templates
        .iter()
        .map(|t| t.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    Err(format!(
        "template '{}' not found, valid templates: {}",
        name, names
    ))
}

fn get_template_name(name: Option<String>) -> String {
    if let Some(name) = name {
        return name;
    }
    let templates = default_templates();
    match Select::new("Pick a template to start your project:", templates).prompt() {
        Ok(template) => template.name,
        Err(_err) => {
            //eprintln!("invalid template name: {}", err);
            std::process::exit(1);
        }
    }
}

fn create_project(name: &str, template: &str, desc: &str) -> anyhow::Result<()> {
    // if directory is exist, print error and return
    let dir = std::path::Path::new(name);
    if dir.exists() {
        cprintln!("<red>project {} already exists</>", name);
        // return Ok(());
    } else {
        // create new directory for project
        std::fs::create_dir(name)?;
    }

    TemplateAssets::iter().for_each(|t| {
        if !t.starts_with(template) {
            return;
        }
        let target = t.replace(template, name);
        let basename = std::path::Path::new(&target).file_name().unwrap();
        let file = TemplateAssets::get(&t).unwrap();
        let content = std::str::from_utf8(&file.data).unwrap().to_string();
        match basename.to_str().unwrap() {
            "Cargo.toml.txt" => handle_cargo_toml(name, template, &target, content).unwrap(),
            "land.toml" => handle_land_toml(name, template, &target, desc, content).unwrap(),
            _ => {
                let parent_dir = std::path::Path::new(&target).parent().unwrap();
                if !parent_dir.exists() {
                    std::fs::create_dir_all(parent_dir).unwrap();
                }
                std::fs::write(target, content).unwrap();
            }
        }
    });

    Ok(())
}

fn handle_cargo_toml(
    name: &str,
    template: &str,
    target: &str,
    content: String,
) -> anyhow::Result<()> {
    let sdk_version =
        std::env::var("SDK_VERSION").unwrap_or(format!("\"{}\"", env!("CARGO_PKG_VERSION")));
    let target = target.replace("Cargo.toml.txt", "Cargo.toml");
    let mut contents = content.replace("{{sdk_version}}", &sdk_version);
    contents = contents.replace(template, name);
    std::fs::write(target, contents)?;
    Ok(())
}

fn handle_land_toml(
    name: &str,
    template: &str,
    target: &str,
    desc: &str,
    content: String,
) -> anyhow::Result<()> {
    let mut contents = content.replace(template, name);
    let template_wasm_name = format!("{}.wasm", template).replace('-', "_");
    let target_wasm_name = format!("{}.wasm", name).replace('-', "_");
    contents = contents.replace(&template_wasm_name, &target_wasm_name);
    contents = contents.replace("\"description\"", desc);
    std::fs::write(target, contents)?;
    Ok(())
}
