use super::params::{CreateTokenRequest, RemoveTokenRequest, TokenResponse};
use super::SessionUser;
use crate::apiv2::RouteError;
use anyhow::Result;
use axum::extract::Query;
use axum::{Extension, Json};
use hyper::StatusCode;
use land_dao::user_token;
use tracing::info;
use validator::Validate;

/// list_handler lists all tokens of current user.
#[tracing::instrument(name = "[tokens_list_handler]", skip_all)]
pub async fn list_handler(
    Extension(current_user): Extension<SessionUser>,
) -> Result<(StatusCode, Json<Vec<TokenResponse>>), RouteError> {
    let tokens =
        user_token::list_by_created(current_user.id, user_token::CreatedByCases::Deployment)
            .await?;
    let values = TokenResponse::from_vec(tokens);
    info!(
        "success, count:{}, userid: {}",
        values.len(),
        current_user.id
    );
    Ok((StatusCode::OK, Json(values)))
}

/// create_for_deployment creates a new token for current user for deployment
#[tracing::instrument(name = "[token_create_handler]", skip_all)]
pub async fn create_handler(
    Extension(current_user): Extension<SessionUser>,
    Json(payload): Json<CreateTokenRequest>,
) -> Result<(StatusCode, Json<TokenResponse>), RouteError> {
    payload.validate()?;

    let token = user_token::find_by_name(
        current_user.id,
        payload.name.clone(),
        user_token::CreatedByCases::Deployment,
    )
    .await?;
    if token.is_some() {
        return Err(anyhow::anyhow!("token name is exist").into());
    }

    let token = user_token::create(
        current_user.id,
        payload.name.clone(),
        365 * 24 * 60 * 60, // 1 year
        user_token::CreatedByCases::Deployment,
    )
    .await?;

    info!(
        "create_for_deployment success, userid:{}, name:{}",
        current_user.id, payload.name
    );
    let mut response = TokenResponse::from_model(&token);
    response.value = token.value;
    Ok((StatusCode::OK, Json(response)))
}

#[tracing::instrument(name = "[token_remove_handler]", skip_all)]
pub async fn remove_handler(
    Extension(current_user): Extension<SessionUser>,
    Query(payload): Query<RemoveTokenRequest>,
) -> Result<(), RouteError> {
    user_token::remove(current_user.id, &payload.uuid).await?;
    info!(
        "remove_token success, userid:{}, uuid:{}",
        current_user.id, payload.uuid
    );
    Ok(())
}
