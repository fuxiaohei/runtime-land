use anyhow::anyhow;
use axum::{
    extract::{Query, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use axum_template::RenderHtml;
use base64::{engine::general_purpose, Engine};
use land_dao::{user_info, user_token};
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{redirect_response, tpls::TemplateEngine, PageVars, ServerError};

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

static CLERK_ENV: OnceCell<ClerkEnv> = OnceCell::new();

/// init_clerk_env initializes ClerkEnv from environment variables
pub fn init_clerk_env() -> anyhow::Result<()> {
    let clerk_env = ClerkEnv {
        publishable_key: std::env::var("CLERK_PUBLISHABLE_KEY").unwrap_or_default(),
        secret_key: std::env::var("CLERK_SECRET_KEY").unwrap_or_default(),
        javascript_src: std::env::var("CLERK_JAVASCRIPT_SRC").unwrap_or_default(),
    };
    info!("ClerkEnv: {:?}", clerk_env);
    CLERK_ENV
        .set(clerk_env)
        .map_err(|_| anyhow!("ClerkEnv is already set"))?;
    Ok(())
}

/// sign_in is a handler for GET /sign-in
pub async fn sign_in(engine: TemplateEngine) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        clerk: ClerkEnv,
    }
    Ok(RenderHtml(
        "sign-in.hbs",
        engine,
        Vars {
            page: PageVars::new("Sign In", "/sign-in", ""),
            clerk: CLERK_ENV.get().unwrap().clone(),
        },
    ))
}

#[derive(Deserialize, Debug)]
pub struct SignCallbackQuery {
    #[serde(rename = "v")]
    pub value: String,
}

impl SignCallbackQuery {
    pub fn to_value(&self) -> anyhow::Result<user_token::SignCallbackValue> {
        let data = general_purpose::STANDARD.decode(&self.value)?;
        let value: user_token::SignCallbackValue = serde_json::from_slice(&data)?;
        Ok(value)
    }
}

/// sign_callback is a handler for GET /sign-callback
pub async fn sign_callback(
    jar: CookieJar,
    Query(query): Query<SignCallbackQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let callback_value = query.to_value().map_err(|e| {
        warn!("Sign callback failed: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    debug!("Sign callback value: {:?}", callback_value);

    // check Clerk session validation
    let clerk_session = jar.get("__session").unwrap();
    let _ = match verify_clerk_session(&callback_value.session_id, clerk_session.value()).await {
        Ok(resp) => resp,
        Err(e) => {
            warn!("Clerk session validation failed: {}", e);
            // sign failed, redirect to sign-out page
            let resp = redirect_response("/sign-out");
            return Ok((jar, resp).into_response());
        }
    };

    let session_token = user_token::create_session(&callback_value)
        .await
        .map_err(|e| {
            warn!("Create session failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let mut session_cookie = Cookie::new("__runtime_land_session", session_token.value);
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));

    let resp = redirect_response("/");
    Ok((jar.add(session_cookie), resp).into_response())
}

/// sign_out is a handler for GET /sign-out
pub async fn sign_out(
    jar: CookieJar,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    let session_value = jar
        .get("__runtime_land_session")
        .map(|c| c.value())
        .unwrap_or_default();
    info!("Remove session: {}", session_value);
    if !session_value.is_empty() {
        let token =
            user_token::get_by_value(session_value, Some(user_token::Usage::Session)).await?;
        if let Some(token) = token {
            user_token::remove(token.id).await?;
        }
    }
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        clerk: ClerkEnv,
    }
    let resp = RenderHtml(
        "sign-out.hbs",
        engine,
        Vars {
            page: PageVars::new("Sign Out", "/sign-out", ""),
            clerk: CLERK_ENV.get().unwrap().clone(),
        },
    )
    .into_response();
    Ok((jar.remove(Cookie::from("__runtime_land_session")), resp).into_response())
}

pub async fn middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();

    // skip static assets auth
    if path.starts_with("/static/") {
        // debug!("auth skip path: {}", path);
        return Ok(next.run(request).await);
    }

    // get session cookie
    let headers = request.headers();
    let jar = CookieJar::from_headers(headers);
    let session_value = jar
        .get("__runtime_land_session")
        .map(|c| c.value())
        .unwrap_or_default();

    // if path is /sign-*, it need validate session
    // if success, /sign-in redirects to homepage, /sign-out continues
    if path.starts_with("/sign") {
        // if session is exist, validate session
        if path.starts_with("/sign-in") && !session_value.is_empty() {
            debug!(path = path, "Session is exist when sign-in");
            let user = verify_runtimeland_session(session_value).await;
            if user.is_ok() {
                // session is ok, redirect to homepage
                return Ok(redirect_response("/").into_response());
            }
        }
        return Ok(next.run(request).await);
    }

    // get clerk session
    let clerk_session = jar.get("__session").map(|c| c.value()).unwrap_or_default();
    if session_value.is_empty() || clerk_session.is_empty() {
        warn!(path = path, "Session or Clerk session is empty");
        // no session, redirect to sign-in page
        return Ok(redirect_response("/sign-in").into_response());
    }

    // after validation, it gets session user from session_id and set to request extensions
    let session_user = verify_runtimeland_session(session_value).await;
    if session_user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page
        return Ok(redirect_response("/sign-out").into_response());
    }
    // debug!(path = path, "Session is valid: {}", session_value);
    request.extensions_mut().insert(session_user.unwrap());
    Ok(next.run(request).await)
}

#[derive(Clone, Serialize, Debug)]
pub struct SessionUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub gravatar: String,
    pub is_admin: bool,
}

async fn verify_runtimeland_session(session_value: &str) -> anyhow::Result<SessionUser> {
    let token = user_token::get_by_value(session_value, Some(user_token::Usage::Session)).await?;
    if token.is_none() {
        return Err(anyhow!("Session not found"));
    }
    let token = token.unwrap();
    let user = user_info::get_by_id(token.user_id, Some(user_info::Status::Active)).await?;
    if user.is_none() {
        return Err(anyhow!("User not found"));
    }
    let user = user.unwrap();
    Ok(SessionUser {
        id: user.id,
        name: user.nick_name,
        email: user.email,
        gravatar: user.gravatar,
        is_admin: user.role == user_info::Role::Admin.to_string(),
    })
}

#[derive(Serialize)]
struct ClerkVerifySessionRequest {
    token: String,
}

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

async fn verify_clerk_session(
    session_id: &str,
    session: &str,
) -> anyhow::Result<ClerkVerifySessionResponse> {
    let verify_api = format!("https://api.clerk.dev/v1/sessions/{}/verify", session_id);
    let verify_data = ClerkVerifySessionRequest {
        token: session.to_string(),
    };
    debug!("Verify clerk session api: {}", verify_api);

    let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
    let client = Client::new();
    let resp = client
        .post(&verify_api)
        .header("User-Agent", user_agent)
        .header(
            "Authorization",
            "Bearer sk_test_mTylRXqX3ds2ZWPo2P3amunjDypN7B7Q6hxqjdEEbD",
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
    let resp: ClerkVerifySessionResponse = resp.json().await?;
    debug!("Verify clerk session response: {:?}", resp);
    Ok(resp)
}
