use super::{dashboard::SessionUser, templates::TemplateEngine, ServerError};
use crate::server::{dashboard::TokenVar, templates::RenderHtmlMinified, PageVars};
use anyhow::Result;
use axum::{response::IntoResponse, Extension};
use axum_csrf::CsrfToken;
use land_dao::models::worker::Model as WorkerModel;
use land_dao::user::TokenUsage;

mod tokens;
pub use tokens::*;

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
        tokens: Vec<TokenVar>,
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

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "admin/index.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Dashboard", "admin-dashboard"),
                user,
                csrf,
                tokens,
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
        workers: Vec<WorkerModel>,
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
    let workers = land_dao::worker::list_all().await?;

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
