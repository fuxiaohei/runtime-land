use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use serde::Serialize;
use tracing::info;

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

/// CLERK_ENV is the environment variables for Clerk.js
pub static CLERK_ENV: OnceCell<ClerkEnv> = OnceCell::new();

/// init_clerk_env initializes ClerkEnv from environment variables
pub async fn init_clerk_env() -> Result<()> {
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
    CLERK_ENV
        .set(clerk_env)
        .map_err(|_| anyhow!("ClerkEnv is already set"))?;

    // init jwks
    // init_clerk_jwks().await?;

    Ok(())
}

/// get_clerk_env returns ClerkEnv
pub fn get_clerk_env() -> ClerkEnv {
    CLERK_ENV.get().unwrap().clone()
}
