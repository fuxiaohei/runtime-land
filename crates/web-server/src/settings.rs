use crate::AppError;
use crate::{sign::SessionUser, PageVars, RenderEngine};
use anyhow::anyhow;
use axum::{response::IntoResponse, Extension};
use axum::{Form, Json};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use validator::Validate;

/// index is the handler for /settings
pub async fn index(
    engine: RenderEngine,
    csrf_token: CsrfToken,
    Extension(user): Extension<SessionUser>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
        pub csrf_token: String,
        pub tokens: Vec<land_dblayer::models::user_token::Model>,
    }

    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let tokens = land_dblayer::user::list_tokens_by_owner(
        user.id,
        land_dblayer::user::TokenCreatedByCases::AccessToken,
    )
    .await?;
    debug!(
        "list_tokens_by_owner, tokens: {:?}, user: {:?}",
        tokens.len(),
        user.id
    );
    Ok((
        csrf_token,
        RenderHtml(
            "settings.hbs",
            engine,
            Vars {
                page: PageVars::new("Settings", "/settings"),
                user,
                csrf_token: csrf_token_value,
                tokens,
            },
        ),
    )
        .into_response())
}

lazy_static! {
    // TOKEN_NAME_REGEX rule is at least 3 characters, alphanumericm, "-" and "_" with no spaces
    static ref TOKEN_NAME_REGEX: Regex = Regex::new(r"^[\w-]{3,}$").unwrap();
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct CreateTokenForm {
    #[validate(regex(
        path = "TOKEN_NAME_REGEX",
        message = "token name need at least 3 characters, alphanumericm, '-' and '_' with no spaces"
    ))]
    pub name: String,
    pub csrf_token: String,
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
    name: String,
    value: String,
}

/// create_token is the handler for /settings/create-token
pub async fn create_token(
    csrf_token: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<CreateTokenForm>,
) -> Result<Json<CreateTokenResponse>, AppError> {
    csrf_token.verify(&form.csrf_token)?;
    form.validate()?;

    // check token name exist
    let token = land_dblayer::user::find_token_by_name(user.id, &form.name).await?;
    if token.is_some() {
        warn!("token name is exist, name: {}", form.name);
        return Err(anyhow!("token name is exist").into());
    }

    let token = land_dblayer::user::create_token(
        user.id,
        &form.name,
        3600 * 24 * 365,
        land_dblayer::user::TokenCreatedByCases::AccessToken,
    )
    .await?;
    info!("create new token: {:?}", token);
    Ok(Json(CreateTokenResponse {
        name: token.name,
        value: token.value,
    }))
}
