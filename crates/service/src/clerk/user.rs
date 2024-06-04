use anyhow::Result;
use clerk_rs::apis::users_api::User;
use clerk_rs::clerk::Clerk;
use clerk_rs::ClerkConfiguration;

/// get_user returns a user by user_id
pub async fn get_user(user_id: &str) -> Result<clerk_rs::models::User> {
    let envs = super::get_env();
    let config = ClerkConfiguration::new(None, None, Some(envs.secret_key), None);
    let client = Clerk::new(config);
    let user = User::get_user(&client, user_id).await?;
    Ok(user)
}
