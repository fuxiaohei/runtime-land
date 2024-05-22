use super::verify_session;
use crate::httputil::response_redirect;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use land_dao::user::UserRole;
use reqwest::StatusCode;
use serde::Serialize;
use tracing::{debug, warn};

#[derive(Clone, Serialize, Debug)]
pub struct SessionUser {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub gravatar: String,
    pub admin: bool,
}

/// middleware is a middleware for session auth
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
            let user = verify_session(session_value).await;
            if user.is_ok() {
                // session is ok, redirect to homepage
                return Ok(response_redirect("/").into_response());
            }
        }
        return Ok(next.run(request).await);
    }

    // get clerk session
    let clerk_session = jar.get("__session").map(|c| c.value()).unwrap_or_default();
    if session_value.is_empty() || clerk_session.is_empty() {
        warn!(path = path, "Session or Clerk session is empty");
        // no session, redirect to sign-in page
        return Ok(response_redirect("/sign-in").into_response());
    }

    // after validation, it gets session user from session_id and set to request extensions
    let user = verify_session(session_value).await;
    if user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page
        return Ok(response_redirect("/sign-out").into_response());
    }
    let user = user.unwrap();
    let session_user = SessionUser {
        id: user.id,
        uuid: user.uuid,
        name: user.nick_name,
        email: user.email,
        gravatar: user.gravatar,
        admin: user.role == UserRole::Admin.to_string(),
    };

    // debug!(path = path, "Session is valid: {}", session_value);
    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}

/// admin_middleware is a middleware for session auth
pub async fn admin_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
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
            let user = verify_session(session_value).await;
            if user.is_ok() {
                // session is ok, redirect to homepage
                return Ok(response_redirect("/").into_response());
            }
        }
        return Ok(next.run(request).await);
    }

    // get clerk session
    let clerk_session = jar.get("__session").map(|c| c.value()).unwrap_or_default();
    if session_value.is_empty() || clerk_session.is_empty() {
        warn!(path = path, "Session or Clerk session is empty");
        // no session, redirect to sign-in page
        return Ok(response_redirect("/sign-in").into_response());
    }

    // after validation, it gets session user from session_id and set to request extensions
    let user = verify_session(session_value).await;
    if user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page
        return Ok(response_redirect("/sign-out").into_response());
    }
    let user = user.unwrap();
    let session_user = SessionUser {
        id: user.id,
        uuid: user.uuid,
        name: user.nick_name,
        email: user.email,
        gravatar: user.gravatar,
        admin: user.role == UserRole::Admin.to_string(),
    };
    if !session_user.admin {
        warn!(path = path, "Session is not admin: {}", session_value);
        return Ok(response_redirect("/sign-in").into_response());
    }

    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}
