use anyhow::{anyhow, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use land_dao::settings;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

static CLERK_JWKS_API: &str = "https://api.clerk.dev/v1/jwks";
static USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
static CLERK_JWKS: &str = "clerk_jwks";

pub async fn request() -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(CLERK_JWKS_API)
        .header("User-Agent", USER_AGENT)
        .header(
            "Authorization",
            format!("Bearer {}", super::ENV.get().unwrap().secret_key),
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
    debug!("Clerk-get-jwks success");
    let text = resp.text().await?;
    Ok(text)
}

#[instrument("[CLERK]")]
pub async fn init() -> Result<()> {
    let value: Option<land_dao::models::settings::Model> = settings::get(CLERK_JWKS).await?;
    if value.is_none() {
        let jwks = request().await?;
        settings::set(CLERK_JWKS, &jwks).await?;
        debug!("JWKS initialized");
    } else {
        debug!("JWKS already initialized");
    }
    Ok(())
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

/// verify verifies the session token
pub async fn verify(session: &str) -> Result<ClerkJwtSession> {
    let j = load().await?;
    if j.alg != "RS256" {
        return Err(anyhow!("JWK key alg is not RS256"));
    }
    let decoding_key = DecodingKey::from_rsa_components(&j.n, &j.e)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    let decoded_token =
        jsonwebtoken::decode::<ClerkJwtSession>(session, &decoding_key, &validation)?;
    Ok(decoded_token.claims)
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

async fn load() -> Result<JwksKey> {
    let setting = settings::get("clerk_jwks").await?;
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
