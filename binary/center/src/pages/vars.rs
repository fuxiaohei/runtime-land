use super::auth::SessionUser;
use crate::settings;
use anyhow::Result;
use chrono::Duration;
use land_dao::deployment::{self, Status};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Add;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageVars {
    pub title: String,
    pub base_uri: String,
    pub version: String,
    pub build_time: String,
}

impl PageVars {
    pub fn new(title: String, base_uri: String) -> Self {
        Self {
            title,
            base_uri,
            version: land_core::version::get_full().to_string(),
            build_time: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserVars {
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub is_admin: bool,
}

impl UserVars {
    pub fn new(user: &SessionUser) -> Self {
        Self {
            name: user.name.clone(),
            email: user.email.clone(),
            avatar: user.avatar.clone(),
            is_admin: user.is_admin,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectVars {
    pub name: String,
    pub language: String,
    pub uuid: String,
    pub deployments: usize,
    pub production_url: String,
    pub production_label: String,
    pub deployment_url: String,
    pub deployment_label: String,
    pub updated_timeago: String,
    pub status_label: String,
    pub prod_domain: String,
    pub prod_protocol: String,
    pub is_inactive: bool,
}

impl ProjectVars {
    pub async fn from_model(project: &land_dao::Project) -> Result<ProjectVars> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let duration = chrono::Utc::now().signed_duration_since(project.updated_at);
        let mut project_vars = ProjectVars {
            name: project.name.clone(),
            language: project.language.clone(),
            uuid: project.uuid.clone(),
            deployments: 0,
            deployment_url: "".to_string(),
            deployment_label: "".to_string(),
            production_url: "".to_string(),
            production_label: "".to_string(),
            updated_timeago: tago.convert(duration.to_std().unwrap()),
            status_label: "".to_string(),
            prod_domain: prod_domain.clone(),
            prod_protocol: prod_protocol.clone(),
            is_inactive: project.status == Status::InActive.to_string(),
        };
        if project.prod_deploy_id > 0 {
            project_vars.production_url =
                format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
            project_vars.production_label = format!("{}.{}", project.name, prod_domain);
            let deployment = deployment::find_by_id(project.owner_id, project.prod_deploy_id)
                .await?
                .unwrap();
            project_vars.deployment_url =
                format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain);
            project_vars.deployment_label = format!("{}.{}", deployment.domain, prod_domain);
        }

        // if project is inactive, no production url
        if project_vars.is_inactive {
            project_vars.status_label = "inactive".to_string();
            project_vars.production_url = "".to_string();
            project_vars.production_label = "".to_string();
        }

        Ok(project_vars)
    }

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
                uuid: project.uuid.clone(),
                deployments: *counter,
                production_url: "".to_string(),
                deployment_url: "".to_string(),
                deployment_label: "".to_string(),
                production_label: "".to_string(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status_label: "running".to_string(),
                prod_domain: prod_domain.clone(),
                prod_protocol: prod_protocol.clone(),
                is_inactive: project.status == Status::InActive.to_string(),
            };
            if project.prod_deploy_id > 0 {
                project_vars.production_url =
                    format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                project_vars.production_label = format!("{}.{}", project.name, prod_domain)
            } else {
                project_vars.status_label = "develop".to_string();
            }
            if *counter == 0 {
                project_vars.status_label = "empty".to_string();
            }

            // if project is inactive, no production url
            if project_vars.is_inactive {
                project_vars.status_label = "inactive".to_string();
                project_vars.production_url = "".to_string();
                project_vars.production_label = "".to_string();
            }

            vars.push(project_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentVars {
    pub domain: String,
    pub prod_domain: String,
    pub deployment_url: String,
    pub deployment_label: String,
    pub is_prod: bool,
    pub is_enabled: bool,
    pub status: String,
    pub deploy_status: String,
    pub updated_timeago: String,
    pub uuid: String,
}

impl DeploymentVars {
    pub async fn from_models(
        deployments: &Vec<land_dao::Deployment>,
    ) -> Result<Vec<DeploymentVars>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for deployment in deployments {
            let duration = chrono::Utc::now().signed_duration_since(deployment.updated_at);
            let deployment_vars = DeploymentVars {
                domain: deployment.domain.clone(),
                prod_domain: deployment.prod_domain.clone(),
                is_prod: !deployment.prod_domain.is_empty(),
                status: deployment.status.clone(),
                deploy_status: deployment.deploy_status.clone(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                deployment_url: format!(
                    "{}://{}.{}",
                    prod_protocol, deployment.domain, prod_domain
                ),
                deployment_label: format!("{}.{}", deployment.domain, prod_domain),
                is_enabled: deployment.status == Status::Active.to_string(),
                uuid: deployment.uuid.clone(),
            };
            vars.push(deployment_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenVars {
    pub name: String,
    pub uuid: String,
    pub expired_timeago: String,
    pub value: String,
}

fn format_future(duration: Duration) -> String {
    let days = duration.num_days();
    if days > 30 {
        let months = days / 30;
        if months > 1 {
            return format!("{} months", months);
        }
        if months == 1 {
            return "1 month".to_string();
        }
    }
    let weeks = duration.num_weeks();
    if weeks > 1 {
        return format!("{} weeks", weeks);
    }
    if weeks == 1 {
        return "1 week".to_string();
    }
    if days > 1 {
        return format!("{} days", days);
    }
    if days == 1 {
        return "1 day".to_string();
    }
    let hours = duration.num_hours();
    if hours > 1 {
        return format!("{} hours", hours);
    }
    if hours == 1 {
        return "1 hour".to_string();
    }
    let minutes = duration.num_minutes();
    if minutes > 1 {
        return format!("{} minutes", minutes);
    }
    if minutes == 1 {
        return "1 minute".to_string();
    }
    let seconds = duration.num_seconds();
    if seconds > 1 {
        return format!("{} seconds", seconds);
    }
    "immidiately".to_string()
}

impl TokenVars {
    pub async fn from_models(
        tokens: &Vec<land_dao::UserToken>,
        new_uuid: Option<String>,
    ) -> (Vec<TokenVars>, Option<TokenVars>) {
        let mut vars = vec![];
        let mut new_token = None;
        for token in tokens {
            let duration = token
                .expired_at
                .unwrap()
                .signed_duration_since(chrono::Utc::now());
            let mut token_vars = TokenVars {
                name: token.name.clone(),
                uuid: token.uuid.clone(),
                expired_timeago: format_future(duration),
                value: String::new(),
            };
            if let Some(uuid) = &new_uuid {
                if uuid == &token.uuid {
                    token_vars.value = token.value.clone();
                    new_token = Some(token_vars);
                    continue;
                }
            }
            vars.push(token_vars);
        }
        (vars, new_token)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectAdminVars {
    pub name: String,
    pub language: String,
    pub uuid: String,
    pub deployments: usize,
    pub is_prod: bool,
    pub production_url: String,
    pub production_label: String,
    pub updated_timeago: String,
    pub status_label: String,
    pub status: String,
    pub owner_name: String,
    pub owner_email: String,
    pub owner_id: i32,
    pub is_active: bool,
}

impl ProjectAdminVars {
    pub async fn from_models(
        projects: &Vec<land_dao::Project>,
        counters: HashMap<i32, usize>,
        users: HashMap<i32, land_dao::User>,
    ) -> Result<Vec<ProjectAdminVars>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for project in projects {
            let user = users.get(&project.owner_id);
            if user.is_none() {
                continue;
            }
            let user = user.unwrap();

            let counter = counters.get(&project.id).unwrap_or(&0);
            let duration = chrono::Utc::now()
                .signed_duration_since(project.updated_at)
                .add(Duration::seconds(2)); // if duation is zero after updated right now, tago.convert fails

            let mut project_vars = ProjectAdminVars {
                name: project.name.clone(),
                language: project.language.clone(),
                uuid: project.uuid.clone(),
                deployments: *counter,
                production_url: "".to_string(),
                production_label: "".to_string(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status_label: "running".to_string(),
                status: project.status.clone(),
                owner_name: user.nick_name.clone(),
                owner_email: user.email.clone(),
                owner_id: user.id,
                is_prod: project.prod_deploy_id > 0,
                is_active: project.status != Status::InActive.to_string(),
            };
            if project.prod_deploy_id > 0 {
                project_vars.production_url =
                    format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                project_vars.production_label = format!("{}.{}", project.name, prod_domain)
            } else {
                project_vars.status_label = "develop".to_string();
            }
            if *counter == 0 {
                project_vars.status_label = "empty".to_string();
            }
            vars.push(project_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationVars {
    pub current: u64,
    pub all: u64,
    pub prev_url: String,
    pub next_url: String,
    pub links: Vec<PaginationLinkVars>,
    pub has_prev: bool,
    pub has_next: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationLinkVars {
    pub url: String,
    pub page: u64,
    pub active: bool,
}

impl PaginationVars {
    pub fn new(current: u64, all: u64, base_uri: &str) -> Self {
        let sep = if base_uri.contains('?') { "&" } else { "?" };
        let prev = if current > 1 { current - 1 } else { 1 };
        let next = if current < all { current + 1 } else { all };
        let prev_url = format!("{}{}page={}", base_uri, sep, prev);
        let next_url = format!("{}{}page={}", base_uri, sep, next);
        let mut links = vec![];
        for i in 1..=all {
            let url = format!("{}{}page={}", base_uri, sep, i);
            let link = PaginationLinkVars {
                url,
                page: i,
                active: i == current,
            };
            links.push(link);
        }
        Self {
            current,
            all,
            prev_url,
            next_url,
            links,
            has_prev: current > 1,
            has_next: current < all,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployAdminVars {
    pub domain: String,
    pub uuid: String,
    pub language: String,
    pub project_name: String,
    pub is_prod: bool,
    pub visit_url: String,
    pub visit_label: String,
    pub updated_timeago: String,
    pub status: String,
    pub owner_name: String,
    pub owner_email: String,
    pub owner_id: i32,
    pub is_active: bool,
}

impl DeployAdminVars {
    pub async fn from_models(
        deploys: &Vec<land_dao::Deployment>,
        projects: HashMap<i32, land_dao::Project>,
        users: HashMap<i32, land_dao::User>,
    ) -> Result<Vec<DeployAdminVars>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for deploy in deploys {
            let project = projects.get(&deploy.project_id);
            if project.is_none() {
                continue;
            }
            let project = project.unwrap();
            let user = users.get(&project.owner_id);
            if user.is_none() {
                continue;
            }
            let user = user.unwrap();

            let duration = chrono::Utc::now()
                .signed_duration_since(deploy.updated_at)
                .add(Duration::seconds(2)); // if duation is zero after updated right now, tago.convert fails
            let mut project_vars = DeployAdminVars {
                domain: deploy.domain.clone(),
                uuid: deploy.uuid.clone(),
                language: project.language.clone(),
                project_name: project.name.clone(),
                visit_url: String::new(),
                visit_label: String::new(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status: deploy.status.clone(),
                owner_name: user.nick_name.clone(),
                owner_email: user.email.clone(),
                owner_id: user.id,
                is_prod: project.prod_deploy_id == deploy.id,
                is_active: deploy.status == Status::Active.to_string(),
            };
            if project_vars.is_active {
                project_vars.visit_url =
                    format!("{}://{}.{}", prod_protocol, deploy.domain, prod_domain);
                project_vars.visit_label = format!("{}.{}", deploy.domain, prod_domain);
                if project_vars.is_prod {
                    project_vars.visit_url =
                        format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                    project_vars.visit_label = format!("{}.{}", project.name, prod_domain);
                }
            }
            vars.push(project_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserAdminVars {
    pub id: i32,
    pub nick_name: String,
    pub email: String,
    pub avatar: String,
    pub role: String,
    pub status: String,
    pub oauth: String,
    pub created_timeago: String,
    pub projects_count: i32,
    pub deployments_count: i32,
}

impl UserAdminVars {
    pub async fn from_models(
        users: &Vec<land_dao::User>,
        project_counters: HashMap<i32, usize>,
        deploy_counters: HashMap<i32, usize>,
    ) -> Result<Vec<UserAdminVars>> {
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for user in users {
            let duration = chrono::Utc::now()
                .signed_duration_since(user.created_at)
                .add(Duration::seconds(2)); // if duation is zero after updated right now, tago.convert fails
            let user_vars = UserAdminVars {
                id: user.id,
                nick_name: user.nick_name.clone(),
                email: user.email.clone(),
                avatar: user.avatar.clone(),
                role: user.role.clone(),
                status: user.status.clone(),
                oauth: user.oauth_social.clone(),
                created_timeago: tago.convert(duration.to_std().unwrap()),
                projects_count: *project_counters.get(&user.id).unwrap_or(&0) as i32,
                deployments_count: *deploy_counters.get(&user.id).unwrap_or(&0) as i32,
            };
            vars.push(user_vars);
        }
        Ok(vars)
    }
}
