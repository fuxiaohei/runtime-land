use super::auth::SessionUser;
use crate::server::{
    dashboard::vars::TokenVar,
    redirect_response,
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars, ServerError,
};
use axum::{response::IntoResponse, Extension, Form};
use axum_csrf::CsrfToken;
use http::StatusCode;
use land_dao::user::TokenUsage;
use tracing::info;

/// index is a handler for GET /settings
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
        land_dao::user::list_tokens_by_user(user.id, Some(TokenUsage::Cmdline)).await?;
    let mut tokens = vec![];
    for token in token_values {
        let is_new = land_dao::user::is_new_token(token.id).await;
        if is_new {
            land_dao::user::unset_new_token(token.id).await;
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
        csrf_layer,
        RenderHtmlMinified(
            "settings.hbs",
            engine,
            IndexVars {
                page: PageVars::new("Account Settings", "settings"),
                user,
                csrf,
                tokens,
            },
        )
        .into_response(),
    ))
}

#[derive(serde::Deserialize, Debug)]
pub struct CreateTokenForm {
    pub name: String,
    pub csrf: String,
}

/// create_token is a handler for POST /settings/create-token
pub async fn create_token(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Form(form): Form<CreateTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let token =
        land_dao::user::create_new_token(user.id, &form.name, 365 * 24 * 3600, TokenUsage::Cmdline)
            .await?;
    info!("New token created: {:?}", token);
    Ok(redirect_response("/settings"))
}

#[derive(serde::Deserialize, Debug)]
pub struct DeleteTokenForm {
    pub name: String,
    pub csrf: String,
    pub id:i32,
}

/// delete_token is a handler for POST /settings/delete-token
pub async fn delete_token(
    Extension(user): Extension<SessionUser>,
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
    if token.user_id != user.id {
        return Err(ServerError::status_code(
            StatusCode::FORBIDDEN,
            "Token not found",
        ));
    }
    if token.name != form.name {
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Token name not match",
        ));
    }
    info!("Delete token: {:?}", token);
    land_dao::user::remove_token(form.id).await?;
    Ok(redirect_response("/settings"))
}
