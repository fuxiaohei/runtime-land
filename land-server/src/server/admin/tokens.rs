use crate::server::{dashboard::SessionUser, redirect_response, ServerError};
use axum::{response::IntoResponse, Extension, Form};
use axum_csrf::CsrfToken;
use http::StatusCode;
use land_dao::user::TokenUsage;
use tracing::info;

#[derive(serde::Deserialize, Debug)]
pub struct CreateTokenForm {
    pub name: String,
    pub csrf: String,
}

/// create_token is a handler for POST /admin/create-token
pub async fn create_token(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Form(form): Form<CreateTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let token =
        land_dao::user::create_new_token(user.id, &form.name, 365 * 24 * 3600, TokenUsage::Worker)
            .await?;
    info!("New token created: {:?}", token);
    Ok(redirect_response("/admin"))
}

#[derive(serde::Deserialize, Debug)]
pub struct DeleteTokenForm {
    pub name: String,
    pub csrf: String,
    pub id: i32,
}

/// delete_token is a handler for POST /admin/delete-token
pub async fn delete_token(
    csrf_layer: CsrfToken,
    Form(form): Form<DeleteTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let token = land_dao::user::get_token_by_id(form.id).await?;
    if token.is_none(){
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Token not found",
        ));
    }
    let token = token.unwrap();
    if token.name != form.name {
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Token name not match",
        ));
    }
    info!("Delete token: {:?}", token);
    land_dao::user::remove_token(form.id).await?;
    Ok(redirect_response("/admin"))
}
