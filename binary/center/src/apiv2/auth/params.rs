use land_dao::{User, UserToken};
use serde::{Deserialize, Serialize};
use validator::Validate;

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
#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}


#[derive(Deserialize, Debug, Validate)]
pub struct CreateTokenRequest {
    pub name: String,
    pub display_name: String,
    pub email: String,
    pub image_url: String,
    pub oauth_id: String,
    pub oauth_provider: String,
    pub oauth_social: String,
}