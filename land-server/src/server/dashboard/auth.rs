use axum::{extract::Query, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use land_core_service::clerkauth::{get_clerk_env, verify_clerk_and_create_token, ClerkEnv};
use land_core_service::httputil::{response_redirect, ServerError};
use land_core_service::template::{self, PageVars, RenderHtmlMinified};
use land_dao::user::{self, SignCallbackValue};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// sign_in renders the /sign-in page
pub async fn sign_in(engine: template::Engine) -> impl IntoResponse {
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
        let resp = response_redirect("/sign-out");
        return Ok((jar, resp).into_response());
    }
    let session_token = session_token.unwrap();
    let mut session_cookie = Cookie::new("__runtime_land_session", session_token.value);
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));
    let resp = response_redirect("/");
    Ok((jar.add(session_cookie), resp).into_response())
}

/// sign_out is a handler for GET /sign-out
pub async fn sign_out(
    jar: CookieJar,
    engine: template::Engine,
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
