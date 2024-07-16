use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::Serialize;
use tracing::{info, instrument};

/// Clerk vars
static VARS: OnceCell<Vars> = OnceCell::new();

#[derive(Serialize, Clone)]
pub struct Vars {
    pub publishable_key: String,
    pub js_src: String,
    #[serde(skip)]
    pub secret_key: String,
}

impl std::fmt::Debug for Vars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vars")
            .field("publishable_key", &self.publishable_key)
            .field("js_src", &self.js_src)
            .finish()
    }
}

impl Vars {
    pub fn new() -> Result<Self> {
        // read from env
        let publishable_key =
            std::env::var("CLERK_PUBLISHABLE_KEY").expect("CLERK_PUBLISHABLE_KEY must be set");
        let js_src = std::env::var("CLERK_JS_SRC").expect("CLERK_JS_SRC must be set");
        let secret_key = std::env::var("CLERK_SECRET_KEY").expect("CLERK_SECRET_KEY must be set");
        Ok(Vars {
            publishable_key,
            js_src,
            secret_key,
        })
    }
}

#[instrument("[CLERK]")]
pub fn init() -> Result<()> {
    let vars = Vars::new()?;
    info!("init {:?}", vars);
    VARS.set(vars)
        .map_err(|_| anyhow::anyhow!("Failed to set vars"))?;
    Ok(())
}

/// get clerk vars
pub fn get() -> Vars{
    VARS.get().expect("Clerk vars not initialized").clone()
}