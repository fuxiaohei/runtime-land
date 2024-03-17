use super::auth::SessionUser;
use crate::server::{redirect_response, tpls::TemplateEngine, PageVars, ServerError};
use axum::{response::IntoResponse, Extension, Form};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use land_dao::{
    user_token::{self, Usage},
    DateTimeUTC,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::info;

#[derive(Serialize)]
struct TokenVar {
    id: i32,
    name: String,
    value: String,
    is_new: bool,
    updated_at: DateTimeUTC,
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
            updated_at: token.updated_at.and_utc(),
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
    usage: Option<String>,
}

/// create_token is a handler for POST /settings/create-token
pub async fn create_token(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<CreateTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf.verify(&form.csrf)?;
    let usage = form.usage.unwrap_or(user_token::Usage::Cmdline.to_string());
    let token = user_token::create(
        user.id,
        &form.name,
        3600 * 24 * 365, // cli token expire 1 year
        user_token::Usage::from_str(&usage).unwrap(),
    )
    .await?;
    info!("Create new token: {:?}", token);
    if usage == user_token::Usage::Worker.to_string() {
        return Ok(redirect_response("/settings/manage"));
    }
    Ok(redirect_response("/settings"))
}

#[derive(Serialize)]
struct WorkerVar {
    ip: String,
    status: String,
    updated_at: DateTimeUTC,
}

/// manage is a handler for GET /manage
pub async fn manage(
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
        token_usage: String,
        workers: Vec<WorkerVar>,
    }
    let csrf_token = csrf.authenticity_token()?;
    let token_values = user_token::list_by_user(user.id, Some(user_token::Usage::Worker)).await?;
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
            updated_at: token.updated_at.and_utc(),
        });
    }

    // workers
    let workers_data = land_dao::worker::list_all().await?;
    let mut workers = vec![];
    for worker in workers_data {
        workers.push(WorkerVar {
            ip: worker.ip,
            status: worker.status,
            updated_at: worker.updated_at.and_utc(),
        });
    }

    Ok((
        csrf,
        RenderHtml(
            "manage.hbs",
            engine,
            Vars {
                page: PageVars::new("Manage", "/manage", "manage"),
                user,
                csrf_token,
                tokens,
                token_usage: Usage::Worker.to_string(),
                workers,
            },
        ),
    )
        .into_response())
}
