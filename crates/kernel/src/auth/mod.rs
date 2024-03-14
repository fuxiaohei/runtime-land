use anyhow::{anyhow, Result};
use land_dao::{
    models::{user_info::Model as UserModel, user_token::Model as UserTokenModel},
    user_info,
    user_token::{self, SignCallbackValue},
};
use serde::Serialize;

mod clerk;
pub use clerk::{init_clerk_env, verify_clerk_session_jwk, ClerkEnv, ClerkJwtSession, CLERK_ENV};

#[derive(Serialize)]
struct ClerkVerifySessionRequest {
    token: String,
}

/*
#[derive(Debug, Deserialize)]
struct ClerkVerifySessionResponse {
    id: String,
    client_id: String,
    user_id: String,
    status: String,
    last_active_at: u64,
    expire_at: u64,
    abandon_at: u64,
    created_at: u64,
    updated_at: u64,
}
*/

/// verify_clerk_and_create_token verifies clerk session and creates a new token
pub async fn verify_clerk_and_create_token(
    sess_value: String,
    callback_value: &SignCallbackValue,
) -> Result<UserTokenModel> {
    let _ = clerk::verify_clerk_session_jwk(sess_value).await?;
    let token = user_token::create_session(callback_value).await?;
    Ok(token)
}

/// verify_session verifies session token
pub async fn verify_session(session_value: &str) -> Result<UserModel> {
    let token = user_token::get_by_value(session_value, Some(user_token::Usage::Session)).await?;
    if token.is_none() {
        return Err(anyhow!("Session not found"));
    }
    let token = token.unwrap();
    let user = user_info::get_by_id(token.user_id, Some(user_info::Status::Active)).await?;
    if user.is_none() {
        return Err(anyhow!("User not found"));
    }
    Ok(user.unwrap())
}
