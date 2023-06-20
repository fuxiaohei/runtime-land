use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct SignupEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12))]
    pub password: String,
    #[validate(length(min = 4))]
    pub nickname: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
    pub access_token_uuid: String,
    pub display_name: String,
    pub display_email: String,
    pub avatar_url: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12))]
    pub password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginAccessTokenRequest {
    #[validate(length(min = 12))]
    pub access_token: String,
}

#[derive(Serialize, Debug)]
pub struct AccessTokenData {
    pub name: String,
    pub value: String,
    pub origin: String,
    pub uuid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub expired_at: i64,
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateTokenRequest {
    #[validate(length(min = 3))]
    pub name: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct RemoveTokenRequest {
    #[validate(length(min = 12))]
    pub uuid: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct FetchProjectRequest {
    #[validate(length(min = 3))]
    pub name: String,
    pub language: String,
}

#[derive(Serialize, Debug)]
pub struct ProjectData {
    pub name: String,
    pub language: String,
    pub uuid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub prod_deployment: i32,
}
