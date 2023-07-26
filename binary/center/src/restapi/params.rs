use std::collections::HashMap;

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
pub struct LoginTokenRequest {
    #[validate(length(min = 12))]
    pub token: String,
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

#[derive(Deserialize, Debug)]
pub struct IpInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub readme: String,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeData {
    pub hostname: String,
    pub cpu_count: usize,
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub updated_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct SyncData {
    pub localip: IpInfo,
    pub region: String,
    pub runtimes: HashMap<String, RuntimeData>,
}
