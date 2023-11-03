use crate::settings;

use super::auth::SessionUser;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageVars {
    pub title: String,
    pub base_uri: String,
}

impl PageVars {
    pub fn new(title: String, base_uri: String) -> Self {
        Self { title, base_uri }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserVars {
    pub name: String,
    pub email: String,
    pub avatar: String,
}

impl UserVars {
    pub fn new(user: &SessionUser) -> Self {
        Self {
            name: user.name.clone(),
            email: user.email.clone(),
            avatar: user.avatar.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVars {
    pub name: String,
    pub language: String,
    pub deployments: usize,
    pub production_url: String,
    pub production_label: String,
    pub updated_timeago: String,
    pub status_label: String,
}

impl ProjectVars {
    pub async fn from_models(
        projects: &Vec<land_dao::Project>,
        counters: HashMap<i32, usize>,
    ) -> Result<Vec<ProjectVars>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for project in projects {
            let counter = counters.get(&project.id).unwrap_or(&0);
            let duration = chrono::Utc::now().signed_duration_since(project.updated_at);
            let mut project_vars = ProjectVars {
                name: project.name.clone(),
                language: project.language.clone(),
                deployments: *counter,
                production_url: "".to_string(),
                production_label: "".to_string(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status_label: "running".to_string(),
            };
            if project.prod_deploy_id > 0 {
                project_vars.production_url =
                    format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                project_vars.production_label = format!("{}.{}", project.name, prod_domain)
            } else {
                project_vars.status_label = "testing".to_string();
            }
            if *counter == 0 {
                project_vars.status_label = "empty".to_string();
            }
            vars.push(project_vars);
        }
        Ok(vars)
    }
}
