use super::{redirect, ServerError};
use crate::templates::Engine;
use axum::response::IntoResponse;
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use axum_template::RenderHtml;
use land_core::clerk;
use land_dao::{tokens, users};
use land_vars::{BreadCrumbKey, Page};
use serde::Serialize;
use tracing::debug;

pub(crate) async fn sign_in(engine: Engine) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub clerk: clerk::Vars,
    }
    RenderHtml(
        "sign-in.hbs",
        engine,
        Vars {
            page: Page::new("Sign in", BreadCrumbKey::SignIn, None),
            clerk: clerk::get(),
        },
    )
}

pub(crate) async fn callback(jar: CookieJar) -> Result<impl IntoResponse, ServerError> {
    let clerk_session = jar.get("__session").map(|c| c.value()).unwrap_or_default();

    // verify clerk session with jwk keys
    let res = clerk::verify_jwks(clerk_session).await?;
    let oauth_user_id = res.sub;

    // get user by oauth user id
    // generate user if not exists
    let mut user = users::get_by_oauth_id(&oauth_user_id).await?;
    if user.is_none() {
        let is_first = users::is_first().await?;
        let role = if is_first {
            users::UserRole::Admin
        } else {
            users::UserRole::Normal
        };
        let clerk_user = clerk::request_user(&oauth_user_id).await?;
        let name = clerk_user.user_name();
        let nick_name = clerk_user.nick_name();
        let email = clerk_user.email();
        let oauth_provider = clerk_user.oauth_provider();
        let avatar = clerk_user.image_url.unwrap_or_default();
        let u2 = users::create(
            name,
            nick_name,
            email,
            avatar,
            oauth_user_id,
            oauth_provider,
            Some(role),
        )
        .await?;
        debug!("create user: {:?}", u2);
        user = Some(u2);
    }

    let user = user.unwrap();
    // create session token
    let token_name = format!("sess-{}-{}", user.id, chrono::Utc::now().timestamp());
    let token = tokens::create(user.id, &token_name, 3600 * 24, tokens::Usage::Session).await?;

    let mut session_cookie = Cookie::new("__runtime_land_session", token.value);
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));
    session_cookie.set_http_only(true);
    let resp = redirect("/");
    Ok((jar.add(session_cookie), resp).into_response())
}

pub(crate) async fn sign_out(engine: Engine, jar: CookieJar) -> impl IntoResponse {
    let jar = jar.remove("__runtime_land_session");
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub clerk: clerk::Vars,
    }
    let resp = RenderHtml(
        "sign-out.hbs",
        engine,
        Vars {
            page: Page::new("Sign Out", BreadCrumbKey::SignIn, None),
            clerk: clerk::get(),
        },
    );
    (jar, resp).into_response()
}
