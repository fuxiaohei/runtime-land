use super::ServerError;
use crate::{
    dash::error_html,
    templates::{Engine, RenderHtmlMinified},
};
use axum::{response::IntoResponse, Extension, Form};
use axum_htmx::HxRedirect;
use land_dao::tokens::{self, Usage};
use land_vars::{AuthUser, BreadCrumbKey, Page, Token};
use serde::{Deserialize, Serialize};
use tracing::info;

/// index is route of user settings page, /settings
pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub tokens: Vec<Token>,
        pub token_create_url: String,
        pub token_remove_url: String,
    }
    let token_values = tokens::list(Some(user.id), Some(tokens::Usage::Cmdline)).await?;
    Ok(RenderHtmlMinified(
        "settings.hbs",
        engine,
        Vars {
            page: Page::new("Settings", BreadCrumbKey::Settings, Some(user)),
            tokens: Token::new_from_models(token_values),
            token_create_url: "/settings/tokens/create".to_string(),
            token_remove_url: "/settings/tokens/remove".to_string(),
        },
    ))
}

/// TokenForm is the form for creating and removing a new token
#[derive(Deserialize, Debug)]
pub struct TokenForm {
    pub name: String,
    pub id: Option<i32>,
}

/// create_token create a new token for user
pub async fn create_token(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    let exist_token = tokens::get_by_name(&f.name, user.id, Some(Usage::Cmdline)).await?;
    if exist_token.is_some() {
        return Ok(error_html("Token name already exists").into_response());
    }
    let token = tokens::create(user.id, &f.name, 3600 * 24 * 365, Usage::Cmdline).await?;

    info!(
        owner_id = user.id,
        token_name = f.name,
        "Create new token: {:?}",
        token
    );

    let uri = axum::http::Uri::from_static("/settings");
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}

/// remove_token removes a token by id
pub async fn remove_token(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    let token = tokens::get_by_name(&f.name, user.id, Some(Usage::Cmdline)).await?;
    if token.is_none() {
        return Ok(error_html("Token not found").into_response());
    }
    let token = token.unwrap();
    if token.id != f.id.unwrap_or(0) {
        return Ok(error_html("Token id not match").into_response());
    }
    tokens::set_expired(token.id, &f.name).await?;
    info!(
        owner_id = user.id,
        token_name = f.name,
        "Remove token: {}",
        token.id,
    );
    let uri = axum::http::Uri::from_static("/settings");
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}
