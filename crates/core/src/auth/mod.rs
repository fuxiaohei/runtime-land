use anyhow::{anyhow, Result};
use land_dao::{
    models::{user_info::Model as UserModel, user_token::Model as UserTokenModel},
    user::{self, SignCallbackValue, TokenUsage, UserStatus},
};

mod clerk;
pub use clerk::{get_clerk_env, init_clerk_env, ClerkEnv};

/// verify_session verifies session token
pub async fn verify_session(session_value: &str) -> Result<UserModel> {
    let token = user::get_token_by_value(session_value, Some(TokenUsage::Session)).await?;
    if token.is_none() {
        return Err(anyhow!("Session not found"));
    }
    let token = token.unwrap();
    let user = user::get_info_by_id(token.user_id, None).await?;
    if user.is_none() {
        return Err(anyhow!("User not found"));
    }
    let user = user.unwrap();
    if user.status == UserStatus::Disabled.to_string() {
        return Err(anyhow!("User is disabled"));
    }
    Ok(user)
}

/// verify_clerk_and_create_token verifies clerk session and creates a new token
pub async fn verify_clerk_and_create_token(
    sess_value: String,
    callback_value: &SignCallbackValue,
) -> Result<UserTokenModel> {
    let _ = clerk::verify_clerk_session_jwk(sess_value).await?;
    let token = user::create_session_token(callback_value).await?;
    Ok(token)
}
