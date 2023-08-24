use land_dao::{User, UserToken};
use land_storage::{FsConfig, S3Config};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct SignupRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 2))]
    pub nickname: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponseTokenField {
    pub active_at: i64,
    pub active_interval: i64,
    pub expired_at: i64,
    pub uuid: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponseUserField {
    pub avatar_url: String,
    pub email: String,
    pub name: String,
    pub oauth_id: String,
    pub role: String,
    pub oauth_provider: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token: LoginResponseTokenField,
    pub user: LoginResponseUserField,
}

impl LoginResponse {
    pub fn new(user: &User, token: &UserToken) -> Self {
        let t = LoginResponseTokenField {
            active_at: token.updated_at.timestamp(),
            active_interval: 60,
            expired_at: token.expired_at.unwrap().timestamp(),
            uuid: token.uuid.clone(),
            value: token.value.clone(),
        };
        let u = LoginResponseUserField {
            avatar_url: user.avatar.clone(),
            email: user.email.clone(),
            name: user.nick_name.clone(),
            oauth_id: user.oauth_id.clone(),
            role: user.role.clone(),
            oauth_provider: user.oauth_provider.clone(),
        };
        Self { token: t, user: u }
    }
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
    pub deployment_url: String,
    pub status: String,
    pub subdomain: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct CreateDeployRequest {
    #[validate(length(min = 3))]
    pub project_name: String,
    pub project_uuid: String,
    pub deploy_chunk: Vec<u8>,
    pub deploy_content_type: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectOverview {
    pub project: ProjectResponse,
    pub deployments: Option<Vec<DeploymentResponse>>,
    pub deployments_count: usize,
    pub prod_deployment: Option<DeploymentResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectRenameRequest {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegionResponse {
    pub id: i32,
    pub name: String,
    pub key: String,
    pub runtimes: i32,
    pub status: String,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct SettingsDomainRequest {
    pub domain: String,
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsStorageResponse {
    pub storage_type: String,
    pub local: FsConfig,
    pub s3: S3Config,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatsResponse {
    pub deployments: i32,
    pub projects: i32,
    pub users: i32,
    pub regions: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRegionTokenRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgetPasswordRequest {
    pub email: String,
    pub base: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordResponse {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdatePasswordRequest {
    pub new_password: String,
    pub confirm_password: String,
    pub current_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageSettingRequest {
    pub typename: String,
    pub fs: Option<land_storage::FsConfig>,
    pub s3: Option<land_storage::S3Config>,
}
