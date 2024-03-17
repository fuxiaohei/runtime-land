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
        domain: String,
        protocol: String,
        storage: String,
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

    // domain, protocol
    let (domain, protocol) = land_dao::settings::get_domain_settings().await?;
    let storage_setting = land_dao::settings::get("storage").await?;
    let storage_content = if let Some(m) = storage_setting {
        m.value
    } else {
        "unknown".to_string()
    };

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
                domain,
                protocol,
                storage: storage_content,
            },
        ),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct UpdateDomainForm {
    protocol: String,
    domain: String,
    csrf: String,
}

/// update_domain is a handler for POST /settings/update-domain
pub async fn update_domain(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<UpdateDomainForm>,
) -> Result<impl IntoResponse, ServerError> {
    if !user.is_admin {
        return Err(ServerError::forbidden("Permission denied"));
    }
    csrf.verify(&form.csrf)?;
    info!("Update domain settings: {},{}", form.protocol, form.domain);
    land_dao::settings::set_domain_settings(form.domain, form.protocol).await?;
    Ok(redirect_response("/settings/manage"))
}

#[derive(Deserialize)]
pub struct UpdateStorageForm {
    storage: String,
    csrf: String,
}

/// update_storage is a handler for POST /settings/update-storage
pub async fn update_storage(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<UpdateStorageForm>,
) -> Result<impl IntoResponse, ServerError> {
    if !user.is_admin {
        return Err(ServerError::forbidden("Permission denied"));
    }
    csrf.verify(&form.csrf)?;
    info!("Update storage: {}", form.storage);
    land_dao::settings::set("storage", &form.storage).await?;
    Ok(redirect_response("/settings/manage"))
}
