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
    pub access_token: String,
    pub access_token_uuid: String,
    pub nick_name: String,
    pub email: String,
    pub avatar_url: String,
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
