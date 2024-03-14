use anyhow::{anyhow, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
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
    init_clerk_jwks().await?;

    Ok(())
}

/*

https://api.clerk.dev/v1/sessions/{}/verify is Deprecated

#[derive(Serialize)]
struct ClerkVerifySessionRequest {
    token: String,
}

pub async fn verify_session(sess_id: String, sess_value: String) -> Result<()> {
    let verify_api = format!("https://api.clerk.dev/v1/sessions/{}/verify", sess_id);
    let verify_data = ClerkVerifySessionRequest { token: sess_value };
    debug!(api = verify_api, "Verify clerk session");

    let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
    let client = reqwest::Client::new();
    let resp = client
        .post(&verify_api)
        .header("User-Agent", user_agent)
        .header(
            "Authorization",
            format!("Bearer {}", CLERK_ENV.get().unwrap().secret_key),
        )
        .json(&verify_data)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "clerk-verify-session error: {}, {}",
            resp.status(),
            resp.text().await?
        ));
    }
    Ok(())
}
*/

async fn init_clerk_jwks() -> Result<()> {
    let value = land_dao::settings::get("clerk_jwks").await?;
    if value.is_none() {
        let jwks = request_jwks().await?;
        land_dao::settings::set("clerk_jwks", &jwks).await?;
    }
    Ok(())
}

pub async fn request_jwks() -> Result<String> {
    let jwks_api = "https://api.clerk.dev/v1/jwks";
    let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
    let client = reqwest::Client::new();
    let resp = client
        .get(jwks_api)
        .header("User-Agent", user_agent)
        .header(
            "Authorization",
            format!("Bearer {}", CLERK_ENV.get().unwrap().secret_key),
        )
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "clerk-get-jwks error: {}, {}",
            resp.status(),
            resp.text().await?
        ));
    }
    let text = resp.text().await?;
    Ok(text)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwksKey {
    #[serde(rename = "use")]
    pub use_key: String,
    pub kty: String,
    pub kid: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwksModel {
    pub keys: Vec<JwksKey>,
}

/// get_jwks returns the first jwks key
async fn get_jwks() -> Result<JwksKey> {
    let setting = land_dao::settings::get("clerk_jwks").await?;
    if setting.is_none() {
        return Err(anyhow!("Clerk_jwks not found"));
    }
    let settings = setting.unwrap();
    let jwks: JwksModel = serde_json::from_str(&settings.value)?;
    if jwks.keys.is_empty() {
        return Err(anyhow!("Clerk_jwks is empty"));
    }
    Ok(jwks.keys[0].clone())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ClerkJwtSession {
    pub azp: String,
    pub exp: i32,
    pub iat: i32,
    pub iss: String,
    pub nbf: i32,
    pub sid: String,
    pub sub: String,
}

/// verify_clerk_session_jwk verifies session token with jwk
pub async fn verify_clerk_session_jwk(session: String) -> Result<ClerkJwtSession> {
    let j = get_jwks().await?;
    if j.alg != "RS256" {
        return Err(anyhow!("JWK key alg is not RS256"));
    }
    let decoding_key = DecodingKey::from_rsa_components(&j.n, &j.e)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    let decoded_token =
        jsonwebtoken::decode::<ClerkJwtSession>(&session, &decoding_key, &validation)?;
    Ok(decoded_token.claims)
}
