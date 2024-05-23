use anyhow::{anyhow, Result};
use clap::Args;
use inquire::validator::Validation;
use inquire::{CustomUserError, Select, Text};
use tracing::debug;

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
        debug!(
            "Project name: {}, template: {}, desc: {}",
            project_name, template_name, desc
        );

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
            name: "js-hello".to_string(),
            desc: "Simple hello world using Javascript".to_string(),
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
