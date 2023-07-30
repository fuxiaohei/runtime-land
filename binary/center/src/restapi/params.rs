use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct SignupEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 4))]
    pub nickname: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token_value: String,
    pub token_uuid: String,
    pub token_expired_at: i64,
    pub nick_name: String,
    pub email: String,
    pub avatar_url: String,
    pub oauth_id: String,
    pub role: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateOauthTokenRequest {
    pub name: String,
    pub display_name: String,
    pub email: String,
    pub image_url: String,
    pub oauth_id: String,
    pub oauth_provider: String,
    pub oauth_social: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateTokenRequest {
    #[validate(length(min = 3))]
    pub name: String,
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

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct CreateProjectRequest {
    pub name: Option<String>,
    pub prefix: Option<String>,
    pub language: String,
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
    pub status: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectOverview {
    pub project: ProjectResponse,
    pub deployments: Option<Vec<DeploymentResponse>>,
    pub deployments_count: usize,
    pub prod_deployment: Option<DeploymentResponse>,
}
