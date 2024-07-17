use anyhow::{anyhow, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use land_dao::{models::user_info, settings, tokens, users};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

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

static CLERK_JWKS_SETTING_KEY: &str = "clerk-jwks";

#[instrument("[CLERK]")]
pub async fn init() -> Result<()> {
    let vars = Vars::new()?;
    info!("init {:?}", vars);
    VARS.set(vars)
        .map_err(|_| anyhow::anyhow!("Failed to set vars"))?;

    // init jwks keys from clerk.dev api
    let jwks_model: Option<JwksModel> = settings::get(CLERK_JWKS_SETTING_KEY).await?;
    if jwks_model.is_none() {
        let jwks = request_jwks().await?;
        settings::set_value(CLERK_JWKS_SETTING_KEY, &jwks).await?;
        info!("init jwks success");
    } else {
        debug!("jwks already exists");
    }

    Ok(())
}

/// get clerk vars
pub fn get() -> Vars {
    VARS.get().expect("Clerk vars not initialized").clone()
}

/// verify_session checks if the session is valid
pub async fn verify_session(value: &str) -> Result<user_info::Model> {
    let token = tokens::get_by_value(value, Some(tokens::Usage::Session)).await?;
    if token.is_none() {
        return Err(anyhow!("Session not found"));
    }
    let token = token.unwrap();
    let user = users::get_by_id(token.owner_id, None).await?;
    if user.is_none() {
        return Err(anyhow!("User not found"));
    }
    let user = user.unwrap();
    if user.status == users::UserStatus::Disabled.to_string() {
        return Err(anyhow!("User is disabled"));
    }
    Ok(user)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JwksKey {
    #[serde(rename = "use")]
    pub use_key: String,
    pub kty: String,
    pub kid: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JwksModel {
    pub keys: Vec<JwksKey>,
}

async fn request_jwks() -> Result<String> {
    let jwks_api = "https://api.clerk.dev/v1/jwks";
    let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
    let client = reqwest::Client::new();
    let vars = get();
    let resp = client
        .get(jwks_api)
        .header("User-Agent", user_agent)
        .header("Authorization", format!("Bearer {}", vars.secret_key))
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

/// verify_jwks verifies session token with jwk
pub async fn verify_jwks(session: &str) -> Result<ClerkJwtSession> {
    let jwks_model: JwksModel = settings::get(CLERK_JWKS_SETTING_KEY).await?.unwrap();
    let j = jwks_model.keys[0].clone();
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
pub struct EmailAddress {
    pub id: String,
    pub email_address: String,
    pub linked_to: Vec<EmailAddressLinkTo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailAddressLinkTo {
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub image_url: Option<String>,
    pub email_addresses: Option<Vec<EmailAddress>>,
}

impl User {
    pub fn user_name(&self) -> String {
        if let Some(username) = self.username.as_ref() {
            return username.clone();
        }
        self.nick_name()
    }

    pub fn nick_name(&self) -> String {
        match (self.first_name.as_ref(), self.last_name.as_ref()) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            _ => "".to_string(),
        }
    }

    pub fn email(&self) -> String {
        if let Some(email_addresses) = self.email_addresses.as_ref() {
            if let Some(email_address) = email_addresses.first() {
                return email_address.email_address.clone();
            }
        }
        "".to_string()
    }

    pub fn oauth_provider(&self) -> String {
        let mut provider = String::new();
        if let Some(email_addresses) = self.email_addresses.as_ref() {
            if let Some(email_address) = email_addresses.first() {
                if let Some(linked_to) = email_address.linked_to.first() {
                    provider.clone_from(&linked_to.typ);
                }
            }
        }
        format!("clerk@{}", provider)
    }
}

/// request gets the user info from Clerk
pub async fn request_user(user_id: &str) -> Result<User> {
    let jwks_api = format!("https://api.clerk.dev/v1/users/{}", user_id);
    let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
    let client = reqwest::Client::new();
    let vars = get();
    let resp = client
        .get(jwks_api)
        .header("User-Agent", user_agent)
        .header("Authorization", format!("Bearer {}", vars.secret_key))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "clerk-get-user error: {}, {}",
            resp.status(),
            resp.text().await?
        ));
    }
    debug!("Clerk-get-user success");
    let u: User = resp.json().await?;
    Ok(u)
}
