use anyhow::{anyhow, Result};
use land_dao::models::user_token::Model as UserTokenModel;
use land_dao::user::SignCallbackValue;
use once_cell::sync::OnceCell;
use serde::Serialize;

mod jwks;
pub use jwks::verify;

mod user;
use tracing::info;
pub use user::get_user;

mod middleware;
pub use middleware::{middleware, AuthUser};

/// ClerkEnv is the environment variables for Clerk.js
#[derive(Serialize, Clone)]
pub struct ClerkEnv {
    pub publishable_key: String,
    pub secret_key: String,
    pub javascript_src: String,
}

impl std::fmt::Debug for ClerkEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClerkEnv")
            .field("publishable_key", &self.publishable_key)
            .field("javascript_src", &self.javascript_src)
            .finish()
    }
}

/// CLERK_ENV is the environment variables for Clerk.dev
static ENV: OnceCell<ClerkEnv> = OnceCell::new();

/// get_env returns ClerkEnv
pub fn get_env() -> ClerkEnv {
    ENV.get().unwrap().clone()
}

/// init_envs initializes ClerkEnv from environment variables
/// if is_init_jwks is true, it will init jwks from clerk.dev api
pub async fn init_envs(is_init_jwks: bool) -> Result<()> {
    let clerk_env = ClerkEnv {
        publishable_key: std::env::var("CLERK_PUBLISHABLE_KEY").unwrap_or_default(),
        secret_key: std::env::var("CLERK_SECRET_KEY").unwrap_or_default(),
        javascript_src: std::env::var("CLERK_JAVASCRIPT_SRC").unwrap_or_default(),
    };

    // must be set
    if clerk_env.publishable_key.is_empty() || clerk_env.secret_key.is_empty() {
        return Err(anyhow!(
            "CLERK_PUBLISHABLE_KEY or CLERK_SECRET_KEY is empty"
        ));
    }
    // must set js
    if clerk_env.javascript_src.is_empty() {
        return Err(anyhow!("CLERK_JAVASCRIPT_SRC is empty"));
    }
    info!("ClerkEnv: {:?}", clerk_env);
    ENV.set(clerk_env)
        .map_err(|_| anyhow!("ClerkEnv is already set"))?;

    if is_init_jwks {
        // init jwks from clerk.dev api, save to db
        jwks::init().await?;
    }

    Ok(())
}

/// create_session_token creates a new session token
pub async fn create_session_token(user_id: &str) -> Result<UserTokenModel> {
    // get local user data if exist
    let user = land_dao::user::get_info_by_origin_id(user_id).await?;
    if user.is_none() {
        // create new token with new user
        let clerk_user = get_user(user_id).await?;
        let mut callback_value = SignCallbackValue {
            session_id: String::new(),
            avatar_url: clerk_user.image_url.unwrap_or_default(),
            first_name: clerk_user
                .first_name
                .unwrap_or_default()
                .unwrap_or_default(),
            full_name: String::new(),
            user_name: clerk_user.username.unwrap_or_default().unwrap_or_default(),
            email: String::new(),
            origin_user_id: clerk_user.id.unwrap_or_default(),
            // TODO: if we have more than one provider, we need to get the provider from the user data
            origin_provider: "clerk@oauth_github".to_string(),
        };
        if let Some(last_name) = clerk_user.last_name.unwrap_or_default() {
            callback_value.full_name = format!("{} {}", callback_value.first_name, last_name);
        } else {
            callback_value
                .full_name
                .clone_from(&callback_value.first_name);
        }
        if let Some(emails) = clerk_user.email_addresses {
            if !emails.is_empty() {
                let email = &emails[0];
                callback_value.email.clone_from(&email.email_address);
            }
        }
        return land_dao::user::create_session_token(&callback_value).await;
    }
    let user = user.unwrap();
    land_dao::user::create_session_token_by_userid(user.id).await
}
