use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use land_core_service::httputil::ServerJsonError;
use land_core_service::vars::TokenVar;
use land_dao::user::TokenUsage;
use land_service::clerk::{self, AuthUser};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSessionParams {
    pub id: String,
    pub session: String,
    pub user_id: String,
}

/// create_session creates a session token
pub async fn create_session(
    Json(j): Json<CreateSessionParams>,
) -> Result<impl IntoResponse, ServerJsonError> {
    clerk::verify(&j.session).await?;

    let token = clerk::create_session_token(&j.user_id).await?;
    info!("Session created: {:?}", token);
    Ok(Json(token))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenParams {
    pub name: String,
}

/// create creates a new token
pub async fn create(
    Extension(user): Extension<AuthUser>,
    Json(j): Json<CreateTokenParams>,
) -> Result<impl IntoResponse, ServerJsonError> {
    // check token exists
    let token = land_dao::user::get_token_by_name(&j.name, user.id).await?;
    if token.is_some() {
        warn!("Token create, but already exists: {:?}", j);
        return Err(ServerJsonError(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("Token already exists"),
        ));
    }
    // create token
    let token = land_dao::user::create_new_token(
        user.id,
        &j.name,
        3600 * 365 * 24,
        land_dao::user::TokenUsage::Cmdline,
    )
    .await?;
    info!("Token created: {:?}", token);
    Ok(Json(token))
}

/// list lists all tokens by user
pub async fn list(
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerJsonError> {
    let tokens = land_dao::user::list_tokens_by_user(user.id, Some(TokenUsage::Cmdline)).await?;
    let vars = TokenVar::from_models_vec(tokens);
    Ok(Json(vars))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteTokenParams {
    pub name: String,
    pub id: i32,
}

/// delete deletes a token
pub async fn delete(
    Extension(user): Extension<AuthUser>,
    Query(j): Query<DeleteTokenParams>,
) -> Result<impl IntoResponse, ServerJsonError> {
    let token = land_dao::user::get_token_by_id(j.id).await?;
    if token.is_none() {
        warn!("Token delete, but not found: {:?}", j);
        return Err(ServerJsonError(
            StatusCode::NOT_FOUND,
            anyhow::anyhow!("Token not found"),
        ));
    }
    let token = token.unwrap();
    if token.name != j.name {
        warn!("Token delete, but name not match: {:?}, {:?}", token, j);
        return Err(ServerJsonError(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("Token name not match"),
        ));
    }
    if token.user_id != user.id {
        warn!("Token delete, but not owned by user: {:?}, {:?}", token, j);
        return Err(ServerJsonError(
            StatusCode::FORBIDDEN,
            anyhow::anyhow!("Token not owned by user"),
        ));
    }
    info!("Token deleted: {:?}", token);
    land_dao::user::remove_token(j.id).await?;
    Ok(Json(()))
}
