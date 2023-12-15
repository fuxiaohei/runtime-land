use crate::RenderEngine;
use anyhow::Result;
use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use axum_template::RenderHtml;
use base64::engine::general_purpose;
use base64::Engine;
use land_dblayer::user::{CreatedByCases, TokenCreatedByCases};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};

pub async fn signin(engine: RenderEngine) -> impl IntoResponse {
    RenderHtml("sign-in.hbs", engine, &{})
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignCallbackRequest {
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

#[instrument(skip_all)]
pub async fn signcallback(
    jar: CookieJar,
    Path(path): Path<String>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    // decode path string as ClerkCallbackRequest
    let data = match general_purpose::STANDARD.decode(path) {
        Ok(v) => v,
        Err(e) => {
            error!("error: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    let req = match serde_json::from_slice::<SignCallbackRequest>(&data) {
        Ok(v) => v,
        Err(e) => {
            error!("error: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    debug!("req: {:?}", req);

    // verify session from clerk js sdk
    let clerk_session = jar.get("__session").unwrap();
    match clerk_verify_session(&req, clerk_session.value().to_string()).await {
        Ok(_) => {}
        Err(e) => {
            error!("error: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // create runtime.land session
    let redirect_to = req.redirect_to.clone();
    let session_value = match create_session_token(&req).await {
        Ok(v) => v,
        Err(e) => {
            error!("error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    debug!("new-session_id: {}", session_value);
    let mut session_cookie = Cookie::new("__runtime_land_session", session_value);
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));
    Ok((jar.add(session_cookie), Redirect::to(&redirect_to)))
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

async fn clerk_verify_session(req: &SignCallbackRequest, session: String) -> anyhow::Result<()> {
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

async fn create_session_token(req: &SignCallbackRequest) -> anyhow::Result<String> {
    let mut user = land_dblayer::user::find_by_oauth(&req.user_id).await?;
    if user.is_none() {
        // if user not exist, create new user
        debug!("user not exist, create new user, email: {}", req.user_email);
        let user2 = land_dblayer::user::create(
            &req.user_name,
            &req.user_email,
            &req.user_image_url,
            &req.user_id,
            &CreatedByCases::Clerk.to_string(),
            &req.oauth_social,
        )
        .await?;
        user = Some(user2);
    }
    let user = user.unwrap();
    // session-id-timestamp
    let token_name = format!("session-{}-{}", user.id, chrono::Utc::now().timestamp());
    let token = land_dblayer::user::create_token(
        user.id,
        &token_name,
        3600 * 23,
        TokenCreatedByCases::Session,
    )
    .await?;
    Ok(token.value)
}

#[derive(Clone, Debug)]
pub struct SessionUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub is_admin: bool,
}

pub async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
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
        error!("auth-middleware: clerk_session or session_id is empty");
        return Ok(Redirect::to("/sign-in").into_response());
    };
    let session_user = match validate_session(session_id).await {
        Ok(v) => v,
        Err(e) => {
            error!("auth-middleware: {}", e);
            return Ok(Redirect::to("/sign-in").into_response());
        }
    };
    debug!("auth-middleware: session_user: {:?}", session_user);
    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}

async fn validate_session(session_value: &str) -> Result<SessionUser> {
    let token = land_dblayer::user::find_token_by_value(session_value).await?;
    if token.is_none() {
        return Err(anyhow::anyhow!("session not exist"));
    }
    let token = token.unwrap();
    if token.status != land_dblayer::user::Status::Active.to_string() {
        return Err(anyhow::anyhow!("session is not active"));
    }
    if token.created_by != TokenCreatedByCases::Session.to_string() {
        return Err(anyhow::anyhow!("session is not valid"));
    }
    let user = land_dblayer::user::find_by_id(token.owner_id).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("user not exist"));
    }
    let user = user.unwrap();
    if user.status != land_dblayer::user::Status::Active.to_string() {
        return Err(anyhow::anyhow!("user is not active"));
    }
    Ok(SessionUser {
        id: user.id,
        name: user.display_name,
        email: user.email,
        avatar: user.avatar_url,
        is_admin: user.role == land_dblayer::user::Role::Admin.to_string(),
    })
}
