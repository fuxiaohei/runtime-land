use axum::{response::IntoResponse, Extension, Form};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use chrono::NaiveDateTime;
use land_dao::user_token;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::auth::SessionUser;
use crate::{redirect_response, tpls::TemplateEngine, PageVars, ServerError};

#[derive(Serialize)]
struct TokenVar {
    id: i32,
    name: String,
    value: String,
    is_new: bool,
    updated_at: NaiveDateTime,
}

/// index is a handler for GET /settings
pub async fn index(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        csrf_token: String,
        tokens: Vec<TokenVar>,
    }
    let csrf_token = csrf.authenticity_token()?;
    let token_values = user_token::list_by_user(user.id, Some(user_token::Usage::Cmdline)).await?;
    let mut tokens = vec![];
    for token in token_values {
        let is_new = user_token::is_new(token.id).await;
        if is_new {
            user_token::unset_new(token.id).await;
        }
        tokens.push(TokenVar {
            id: token.id,
            name: token.name,
            value: token.value,
            is_new,
            updated_at: token.updated_at,
        });
    }
    Ok((
        csrf,
        RenderHtml(
            "settings.hbs",
            engine,
            Vars {
                page: PageVars::new("Settings", "/settings", ""),
                user,
                csrf_token,
                tokens,
            },
        ),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct CreateTokenForm {
    name: String,
    csrf: String,
}

/// create_token is a handler for POST /settings/create-token
pub async fn create_token(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<CreateTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf.verify(&form.csrf)?;
    let token = user_token::create(
        user.id,
        &form.name,
        3600 * 24 * 365, // cli token expire 1 year
        user_token::Usage::Cmdline,
    )
    .await?;
    info!("Create new token: {:?}", token);
    Ok(redirect_response("/settings"))
}
