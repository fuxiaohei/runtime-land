use super::redirect_response;
use super::{dashboard::SessionUser, templates::TemplateEngine, ServerError};
use crate::server::dashboard::WorkerVar;
use crate::server::{dashboard::TokenVar, templates::RenderHtmlMinified, PageVars};
use anyhow::Result;
use axum::extract::{Query, Request};
use axum::{response::IntoResponse, Extension};
use axum::{Form, Json};
use axum_csrf::CsrfToken;
use http::StatusCode;
use land_dao::user::TokenUsage;
use tracing::info;

mod tokens;
pub use tokens::*;
mod projects;
pub use projects::*;

/// index is a handler for GET /admin/
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
    }

    let csrf = csrf_layer.authenticity_token()?;

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "admin/index.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Dashboard", "admin-dashboard"),
                user,
                csrf,
            },
        ),
    )
        .into_response())
}

/// workers is a handler for GET /admin/workers
pub async fn workers(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
        tokens: Vec<TokenVar>,
        workers: Vec<WorkerVar>,
    }

    let csrf = csrf_layer.authenticity_token()?;

    // list cmd line tokens
    let token_values =
        land_dao::user::list_tokens_by_user(user.id, Some(TokenUsage::Worker)).await?;
    let mut tokens = vec![];
    for token in token_values {
        // need to check if the token is new, unset it if it is
        let is_new = land_dao::user::is_new_token(token.id).await;
        if is_new {
            land_dao::user::unset_new_token(token.id).await;
        }
        tokens.push(TokenVar {
            id: token.id,
            name: token.name,
            value: token.value,
            is_new: true,
            updated_at: token.updated_at.and_utc(),
        });
    }

    // list workers
    let workers_value = land_dao::worker::list_all().await?;
    let workers = WorkerVar::from_models_vec(workers_value);

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "admin/workers.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Workers", "admin-workers"),
                user,
                csrf,
                tokens,
                workers,
            },
        ),
    )
        .into_response())
}

#[derive(serde::Deserialize, Debug)]
pub struct SettingsQuery {
    pub name: Option<String>,
}

/// settings is a handler for GET /admin/settings
pub async fn settings(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: TemplateEngine,
    Query(q): Query<SettingsQuery>,
) -> Result<impl IntoResponse, ServerError> {
    // if name is not None, it means the user is trying to read one setting and return as json not page
    if q.name.is_some() {
        let settings = land_dao::settings::get(&q.name.unwrap()).await?;
        if settings.is_none() {
            return Err(ServerError::status_code(
                StatusCode::NOT_FOUND,
                "Setting not found",
            ));
        }
        return Ok(Json(settings.unwrap()).into_response());
    }
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
        settings: Vec<String>,
    }

    let csrf = csrf_layer.authenticity_token()?;
    let settings = land_dao::settings::list_names().await?;

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "admin/settings.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Settings", "admin-settings"),
                user,
                csrf,
                settings,
            },
        ),
    )
        .into_response())
}

#[derive(serde::Deserialize, Debug)]
pub struct SettingsForm {
    pub name: String,
    pub value: String,
    pub csrf: String,
}

/// settings is a handler for GET /admin/settings
pub async fn update_settings(
    // Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Form(f): Form<SettingsForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&f.csrf)?;
    land_dao::settings::set(&f.name, &f.value).await?;
    info!("Setting updated: {}", f.name);

    // if storage is updated, need reload
    if f.name.eq("storage") {
        info!("Reload storage settings");
        land_dao::settings::reload_storage().await?;
    }

    Ok(redirect_response("/admin/settings"))
}

/// debug is a handler for GET /admin/debug
pub async fn debug(req: Request) -> Result<impl IntoResponse, ServerError> {
    // print uri and headers
    info!("uri: {}", req.uri());
    for (key, value) in req.headers() {
        info!("{}: {:?}", key, value);
    }
    Ok(redirect_response("/admin"))
}
