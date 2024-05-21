use crate::server::{
    redirect_response,
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars, ServerError,
};
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
use land_dao::clerkauth::{get_clerk_env, verify_clerk_and_create_token, verify_session, ClerkEnv};
use land_dao::user::{self, SignCallbackValue, UserRole};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

#[derive(Clone, Serialize, Debug)]
pub struct SessionUser {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub gravatar: String,
    pub admin: bool,
}

/// sign_in renders the /sign-in page
pub async fn sign_in(engine: TemplateEngine) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct SignInVars {
        page: PageVars,
        clerk: ClerkEnv,
    }
    RenderHtmlMinified(
        "auth/sign_in.hbs",
        engine,
        SignInVars {
            page: PageVars::new("Sign-In", ""),
            clerk: get_clerk_env(),
        },
    )
}

#[derive(Deserialize, Debug)]
pub struct SignCallbackQuery {
    #[serde(rename = "v")]
    pub value: String,
}

impl SignCallbackQuery {
    pub fn to_value(&self) -> anyhow::Result<SignCallbackValue> {
        let data = land_common::encoding::base64_decode(&self.value)?;
        let value: SignCallbackValue = serde_json::from_slice(&data)?;
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
    let clerk_session = jar.get("__session").unwrap().value().to_string();
    let session_token = verify_clerk_and_create_token(clerk_session, &callback_value).await;
    if session_token.is_err() {
        warn!(
            "Clerk session validation failed: {}",
            session_token.err().unwrap()
        );
        // sign failed, redirect to sign-out page
        let resp = redirect_response("/sign-out");
        return Ok((jar, resp).into_response());
    }
    let session_token = session_token.unwrap();
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
        user::remove_session_token(session_value).await?;
    }
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        clerk: ClerkEnv,
    }
    let resp = RenderHtmlMinified(
        "auth/sign_out.hbs",
        engine,
        Vars {
            page: PageVars::new("Sign Out", "/sign-out"),
            clerk: get_clerk_env(),
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
            let user = verify_session(session_value).await;
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
    let user = verify_session(session_value).await;
    if user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page
        return Ok(redirect_response("/sign-out").into_response());
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

    // /admin need admin role
    if path.starts_with("/admin") && !session_user.admin {
        warn!(path = path, "Session is not admin: {}", session_value);
        return Ok(redirect_response("/overview").into_response());
    }

    // debug!(path = path, "Session is valid: {}", session_value);
    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}
