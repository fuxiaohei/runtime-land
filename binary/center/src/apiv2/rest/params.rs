use crate::settings;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectOverview {
    pub project: ProjectResponse,
    pub deployments: Option<Vec<DeploymentResponse>>,
    pub deployments_count: usize,
    pub prod_deployment: Option<DeploymentResponse>,
}

impl ProjectOverview {
    pub async fn from_vec(
        projects: Vec<land_dao::Project>,
        counters: HashMap<i32, usize>,
        user_id: i32,
    ) -> Result<Vec<ProjectOverview>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let mut project_overviews = Vec::new();
        for project in projects {
            let counter = counters.get(&project.id).unwrap_or(&0);
            let project_response = ProjectResponse::from_model(&project, &prod_domain);

            let mut overview = ProjectOverview {
                deployments_count: *counter,
                deployments: None,
                prod_deployment: None,
                project: project_response,
            };

            // load prod deployment
            if project.prod_deploy_id > 0 {
                let deployment =
                    land_dao::deployment::find_by_id(user_id, project.prod_deploy_id).await?;
                if deployment.is_some() {
                    let deployment = deployment.unwrap();
                    overview.project.prod_url =
                        format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain);
                    overview.prod_deployment = Some(DeploymentResponse::from_model(
                        &deployment,
                        &prod_domain,
                        &prod_protocol,
                    ));
                }
            }

            project_overviews.push(overview);
        }

        Ok(project_overviews)
    }

    pub async fn from_model(project: &land_dao::Project) -> Result<ProjectOverview> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let project_response = ProjectResponse::from_model(project, &prod_domain);
        let mut overview = ProjectOverview {
            deployments_count: 0,
            deployments: None,
            prod_deployment: None,
            project: project_response,
        };

        let deployments = land_dao::deployment::list_by_project_id(project.id).await?;
        overview.deployments_count = deployments.len();

        let mut deployments_response = vec![];
        for deployment in deployments {
            let deployment_response =
                DeploymentResponse::from_model(&deployment, &prod_domain, &prod_protocol);
            if deployment.id == project.prod_deploy_id {
                overview.project.prod_url = deployment_response.prod_url.clone();
                overview.project.deployment_url = deployment_response.domain_url.clone();
                overview.prod_deployment = Some(deployment_response.clone());
            }
            deployments_response.push(deployment_response);
        }
        overview.deployments = Some(deployments_response);
        Ok(overview)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectResponse {
    pub name: String,
    pub language: String,
    pub uuid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub prod_deployment: i32,
    pub prod_url: String,
    pub deployment_url: String,
    pub status: String,
    pub subdomain: String,
}

impl ProjectResponse {
    pub fn from_model(project: &land_dao::Project, prod_domain: &str) -> ProjectResponse {
        ProjectResponse {
            language: project.language.clone(),
            uuid: project.uuid.clone(),
            prod_deployment: project.prod_deploy_id,
            prod_url: "".to_string(),
            deployment_url: "".to_string(),
            status: project.status.clone(),
            name: project.name.clone(),
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
            subdomain: prod_domain.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeploymentResponse {
    pub id: i32,
    pub project_id: i32,
    pub domain: String,
    pub domain_url: String,
    pub prod_domain: String,
    pub prod_url: String,
    pub uuid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deploy_status: String,
    pub status: String,
}

impl DeploymentResponse {
    pub fn from_model(
        deployment: &land_dao::Deployment,
        prod_domain: &str,
        prod_protocol: &str,
    ) -> DeploymentResponse {
        DeploymentResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            uuid: deployment.uuid.clone(),
            domain: deployment.domain.clone(),
            domain_url: format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain),
            prod_domain: deployment.prod_domain.clone(),
            prod_url: format!(
                "{}://{}.{}",
                prod_protocol, deployment.prod_domain, prod_domain
            ),
            created_at: deployment.created_at.timestamp(),
            updated_at: deployment.updated_at.timestamp(),
            status: deployment.status.clone(),
            deploy_status: deployment.deploy_status.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct CreateProjectRequest {
    pub name: Option<String>,
    pub prefix: Option<String>,
    pub language: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct CreateDeployRequest {
    #[validate(length(min = 3))]
    pub project_name: String,
    pub project_uuid: String,
    pub deploy_chunk: Vec<u8>,
    pub deploy_content_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateDeployRequest {
    pub project_uuid: String,
    pub deployment_uuid: String,
    pub action: String,
}

#[derive(Serialize, Debug)]
pub struct TokenResponse {
    pub name: String,
    pub value: String,
    pub origin: String,
    pub uuid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub expired_at: i64,
}

impl TokenResponse {
    pub fn from_vec(tokens: Vec<land_dao::UserToken>) -> Vec<TokenResponse> {
        let mut values = vec![];
        for token in tokens {
            let value = TokenResponse::from_model(&token);
            values.push(value);
        }
        values
    }

    pub fn from_model(t: &land_dao::UserToken) -> TokenResponse {
        TokenResponse {
            name: t.name.clone(),
            created_at: t.created_at.timestamp(),
            updated_at: t.updated_at.timestamp(),
            expired_at: t.expired_at.unwrap().timestamp(),
            origin: t.created_by.to_string(),
            uuid: t.uuid.clone(),
            value: String::new(),
        }
    }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateTokenRequest {
    #[validate(length(min = 3))]
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct RemoveTokenRequest {
    pub uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectRenameRequest {
    pub old_name: String,
    pub new_name: String,
}
