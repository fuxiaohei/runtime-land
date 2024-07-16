use super::ServerError;
use crate::dash::redirect;
use axum::{
    extract::{ConnectInfo, Request},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use land_core::clerk;
use land_vars::AuthUser;
use std::net::SocketAddr;
use tracing::{debug, info, instrument, warn};

/// SESSION_COOKIE_NAME is the session cookie name
static SESSION_COOKIE_NAME: &str = "__runtime_land_session";

/// auth is a middleware to check if the user is authenticated
pub async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();

    // skip static assets
    if path.starts_with("/static/") {
        // debug!("auth skip path: {}", path);
        return Ok(next.run(request).await);
    }

    // get session cookie
    let headers = request.headers();
    let jar = CookieJar::from_headers(headers);
    let session_value = jar
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value())
        .unwrap_or_default();

    // if path is /sign-*, it need validate session
    // if success, /sign-in redirects to homepage, /sign-out continues
    if path.starts_with("/sign") {
        // if session is exist, validate session in sign-in page
        if path.starts_with("/sign-in") && !session_value.is_empty() {
            debug!(path = path, "Session is exist when sign-in");
            let user = clerk::verify_session(session_value).await;
            if user.is_ok() {
                // session is ok, redirect to homepage
                return Ok(redirect("/").into_response());
            }
        }
        return Ok(next.run(request).await);
    }

    // get clerk session
    let clerk_session = jar.get("__session").map(|c| c.value()).unwrap_or_default();
    if session_value.is_empty() || clerk_session.is_empty() {
        warn!(path = path, "Session or Clerk session is empty");
        if request.method() != "GET" {
            // skip redirect for GET method
            return Err(StatusCode::UNAUTHORIZED);
        }
        // no clerk session, redirect to sign-in page
        return Ok(redirect("/sign-in").into_response());
    }

    // after validation, it gets session user from session_id and set to request extensions
    let user = clerk::verify_session(session_value).await;
    if user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page to remove session
        return Ok(redirect("/sign-out").into_response());
    }

    let user = user.unwrap();
    let session_user = AuthUser::new(&user);
    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}

#[instrument("[HTTP]", skip_all)]
pub async fn logger(request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();
    if path.starts_with("/static") {
        // ignore static assets log
        return Ok(next.run(request).await);
    }
    let mut remote = "0.0.0.0".to_string();
    // if x-real-ip exists, use it
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        remote = real_ip.to_str().unwrap().to_string();
    } else if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        remote = connect_info.to_string();
    }

    /*
    if path.starts_with("/api/v1/worker-api/alive") {
        // high sequence url
        return Ok(next.run(request).await);
    }*/

    let method = request.method().clone().to_string();
    let st = tokio::time::Instant::now();
    let resp = next.run(request).await;
    let server_err = resp.extensions().get::<ServerError>();
    let status = resp.status().as_u16();
    let elasped = st.elapsed().as_millis();
    if let Some(err) = server_err {
        warn!(
            remote = remote,
            method = method,
            path = path,
            status = status,
            elasped = elasped,
            "Failed: {}",
            err.1
        );
    } else {
        let empty_header = HeaderValue::from_str("").unwrap();
        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap();
        let content_size = resp
            .headers()
            .get("content-length")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap();
        if status >= 400 {
            warn!(
                rmt = remote,
                m = method,
                p = path,
                s = status,
                cost = elasped,
                typ = content_type,
                size = content_size,
                "Ok",
            );
        } else {
            info!(
                rmt = remote,
                m = method,
                p = path,
                s = status,
                cost = elasped,
                typ = content_type,
                size = content_size,
                "Ok",
            );
        }
    }
    Ok(resp)
}
