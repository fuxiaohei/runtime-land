use anyhow::{anyhow, Result};
use clap::Args;
use color_print::cprintln;
use inquire::{validator::Validation, CustomUserError, Select, Text};
use land_core::examples::{self, Item};
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

        // get project name, template, desc
        let project_name = get_project_name(self.name.clone())?;
        let tpl = get_template_name(self.template.clone())?;
        let desc = get_desc(self.desc.clone())?;
        debug!(
            "Project name: {}, template: {}, desc: {}",
            project_name, tpl, desc
        );

        cprintln!("<green>Create project '{}'</green>", project_name);

        // check dir named 'project_name' exists
        if !std::path::Path::new(&project_name).exists() {
            // create dir named 'project_name'
            std::fs::create_dir(&project_name)?;
        }

        // extract template to directory as project_name
        cprintln!(
            "<green>Extracting template '{}' to '{}'</green>",
            tpl.title,
            project_name,
        );
        tpl.extract(&project_name, &desc)?;

        // print success message
        cprintln!(
            "<green>Project '{}' created successfully</green>",
            project_name
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

fn get_desc(desc: Option<String>) -> Result<String> {
    if let Some(desc) = desc {
        return Ok(desc);
    }
    match Text::new("Enter a description for your project (Optional):").prompt() {
        Ok(desc) => Ok(desc),
        Err(_err) => Err(anyhow!("Project description is invalid")),
    }
}

fn get_template_name(name: Option<String>) -> Result<Item> {
    let templates = examples::defaults();
    if let Some(name) = name {
        match templates.iter().find(|tpl| tpl.link == name) {
            Some(tpl) => return Ok(tpl.clone()),
            None => return Err(anyhow!("Template not found")),
        }
    }
    match Select::new("Pick a template to start your project:", templates).prompt() {
        Ok(template) => Ok(template),
        Err(_err) => Err(anyhow!("Template not found")),
    }
}
