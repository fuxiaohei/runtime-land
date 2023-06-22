use crate::auth::CurrentUser;
use crate::{params, AppError};
use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::Json;
use land_core::dao;
use tracing::info;
use validator::Validate;

/// list_handler lists all tokens of current user.
pub async fn list_handler(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::AccessTokenData>>), AppError> {
    let tokens = dao::token::list(current_user.id).await?;
    let values: Vec<params::AccessTokenData> = tokens
        .into_iter()
        .map(|t| params::AccessTokenData {
            name: t.name,
            created_at: t.created_at.timestamp(),
            updated_at: t.updated_at.timestamp(),
            expired_at: t.expired_at as i64,
            origin: t.origin,
            uuid: t.uuid,
            value: String::new(),
        })
        .collect();
    info!(
        "list_tokens success, count:{}, userid: {}",
        values.len(),
        current_user.id
    );
    Ok((StatusCode::OK, Json(values)))
}

/// create_handler creates a new token for current user.
pub async fn create_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::CreateTokenRequest>,
) -> Result<(StatusCode, Json<params::AccessTokenData>), AppError> {
    payload.validate()?;
    let tk = dao::token::create(
        current_user.id,
        payload.name.clone(),
        "dashboard".to_string(),
        365 * 24 * 3600,
    )
    .await?;
    info!(
        "create_tokens success, userid:{}, name:{}",
        current_user.id, payload.name
    );
    Ok((
        StatusCode::OK,
        Json(params::AccessTokenData {
            name: tk.name,
            created_at: tk.created_at.timestamp(),
            updated_at: tk.updated_at.timestamp(),
            expired_at: tk.expired_at as i64,
            origin: tk.origin,
            uuid: tk.uuid,
            value: tk.token,
        }),
    ))
}

/// remove_handler removes a token of current user.
pub async fn remove_handler(
    Extension(current_user): Extension<CurrentUser>,
    Query(payload): Query<params::RemoveTokenRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;
    dao::token::remove(current_user.id, payload.uuid.clone()).await?;
    info!(
        "remove_tokens success, userid:{}, uuid:{}",
        current_user.id, payload.uuid
    );
    Ok(StatusCode::OK)
}
