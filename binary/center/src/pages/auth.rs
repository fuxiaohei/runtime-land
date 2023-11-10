use super::AppEngine;
use crate::pages::vars::PageVars;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};
use axum::{middleware::Next, response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use axum_template::RenderHtml;
use base64::{engine::general_purpose, Engine as _};
use hyper::{Request, StatusCode};
use land_dao::user::{create_by_oauth, find_by_oauth_id, OauthProvider, Role};
use land_dao::user_token::{self, CreatedByCases};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub struct SessionUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub is_admin: bool,
}

pub async fn session_auth_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<response::Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();

    // sign-in, sign-callback, sign-out should skip
    // /static/
    if path.starts_with("/sign-") || path.starts_with("/static/") {
        return Ok(next.run(request).await);
    }

    let headers = request.headers();
    let jar = CookieJar::from_headers(headers);
    let session_id = jar
        .get("__runtime_land_session")
        .map(|c| c.value())
        .unwrap_or_default();
    let clerk_session = jar.get("__session").map(|c| c.value()).unwrap_or_default();
    if session_id.is_empty() || clerk_session.is_empty() {
        error!("session-auth-middleware: clerk_session or session_id is empty");
        return Ok(Redirect::to("/sign-in").into_response());
    }
    let (token, user) =
        match user_token::find_by_value_with_active_user(session_id.to_string()).await {
            Ok(v) => v,
            Err(e) => {
                error!("session-auth-middleware error: {}", e);
                return Ok(Redirect::to("/sign-in").into_response());
            }
        };
    if token.created_by != CreatedByCases::Session.to_string() {
        error!("session-auth-middleware: token created by not session");
        return Ok(Redirect::to("/sign-in").into_response());
    }
    let session_user = SessionUser {
        id: user.id,
        name: user.nick_name,
        email: user.email,
        avatar: user.avatar,
        is_admin: user.role == Role::Admin.to_string(),
    };
    debug!("session-auth-middleware: session_user: {:?}", session_user);

    if path.starts_with("/admin") && !session_user.is_admin {
        error!("session-auth-middleware: user role is not admin");
        return Ok(Redirect::to("/projects").into_response());
    }

    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClerkCallbackRequest {
    session_id: String,
    user_id: String,
    user_image_url: String,
    user_first_name: String,
    user_full_name: String,
    user_name: String,
    user_email: String,
    oauth_social: String,
    redirect_to: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClerkCallbackResponse {
    pub ok: bool,
}

#[derive(Serialize, Deserialize)]
struct ClerkVerifySessionRequest {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

async fn clerk_verify_session(req: &ClerkCallbackRequest, session: String) -> anyhow::Result<()> {
    let verify_api = format!(
        "https://api.clerk.dev/v1/sessions/{}/verify",
        req.session_id,
    );
    let verify_data = ClerkVerifySessionRequest { token: session };
    debug!("clerk-verify-session: {}", verify_api);
    let resp = reqwest::Client::new()
        .post(verify_api)
        .header(
            "Authorization",
            "Bearer sk_test_mTylRXqX3ds2ZWPo2P3amunjDypN7B7Q6hxqjdEEbD",
        )
        .json(&verify_data)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "clerk-verify-session error: {}, {}",
            resp.status(),
            resp.text().await?
        ));
    }
    let resp: ClerkVerifySessionResponse = resp.json().await?;
    debug!("clerk-verify-session-resp: {:?}", resp);
    Ok(())
}

pub async fn clerk_callback(
    jar: CookieJar,
    Path(path): Path<String>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    // decode path string as ClerkCallbackRequest
    info!("path: {}", path);
    let data = match general_purpose::STANDARD.decode(path) {
        Ok(v) => v,
        Err(e) => {
            error!("clerk-callback error: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    let req = match serde_json::from_slice::<ClerkCallbackRequest>(&data) {
        Ok(v) => v,
        Err(e) => {
            error!("clerk-callback error: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    let redirect_to = req.redirect_to.clone();
    let clerk_session = jar.get("__session").unwrap();
    match clerk_verify_session(&req, clerk_session.value().to_string()).await {
        Ok(_) => {}
        Err(e) => {
            error!("clerk-callback error: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    let session_id = match create_session_id(&req).await {
        Ok(v) => v,
        Err(e) => {
            error!("clerk-callback error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    debug!("clerk-callback-new-session_id: {}", session_id);
    let mut session_cookie = Cookie::new("__runtime_land_session", session_id);
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));
    Ok((jar.add(session_cookie), Redirect::to(&redirect_to)))
}

async fn create_session_id(req: &ClerkCallbackRequest) -> anyhow::Result<String> {
    let mut user = find_by_oauth_id(req.user_id.clone()).await?;
    // first user login ,create this user
    if user.is_none() {
        debug!(
            "create_session_id: create user by oauth, email:{}",
            req.user_email
        );
        let user2 = create_by_oauth(
            req.user_name.clone(),
            req.user_full_name.clone(),
            req.user_email.clone(),
            req.user_image_url.clone(),
            req.user_id.clone(),
            OauthProvider::Clerk.to_string(),
            req.oauth_social.clone(),
        )
        .await?;
        user = Some(user2);
    }
    let user = user.unwrap();
    let token = user_token::create(
        user.id,
        "dashboard-session".to_string(),
        3600 * 23,
        CreatedByCases::Session,
    )
    .await?;
    Ok(token.value)
}

#[derive(Debug, Serialize, Deserialize)]
struct SigninVars {
    page: PageVars,
}

/// render_signin renders sign-in page
pub async fn render_signin(engine: AppEngine) -> impl IntoResponse {
    let page_vars = PageVars::new("Sign-in".to_string(), String::new());
    let vars = SigninVars { page: page_vars };
    RenderHtml("sign-in.hbs", engine, vars)
}

/// render_signout renders sign-out page
pub async fn render_signout(engine: AppEngine, jar: CookieJar) -> impl IntoResponse {
    let value = jar
        .get("__runtime_land_session")
        .map(|c| c.value())
        .unwrap_or_default();
    if !value.is_empty() {
        debug!("render-signout: remove session_id: {}", value);
        let _ = user_token::remove_by_value(value).await;
    }
    RenderHtml("sign-out.hbs", engine, &())
}
