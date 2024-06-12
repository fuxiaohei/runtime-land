use axum::{extract::Path, response::IntoResponse, Extension, Form};
use axum_csrf::CsrfToken;
use http::StatusCode;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::{response_redirect, ServerError};
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::{PageVars, TokenVar};
use land_dao::{envs::EnvsParams, user::TokenUsage};
use tracing::info;

/// index is a handler for GET /settings
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: template::Engine,
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
            created_at: token.created_at.and_utc(),
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
    Ok(response_redirect("/settings"))
}

#[derive(serde::Deserialize, Debug)]
pub struct DeleteTokenForm {
    pub name: String,
    pub csrf: String,
    pub id: i32,
}

/// delete_token is a handler for POST /settings/delete-token
pub async fn delete_token(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Form(form): Form<DeleteTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let token = land_dao::user::get_token_by_id(form.id).await?;
    if token.is_none() {
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
    Ok(response_redirect("/settings"))
}

/// update_envs is a handler for POST /{project_name}/update-envs
pub async fn update_envs(
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
    axum_extra::extract::Form(form): axum_extra::extract::Form<EnvsParams>,
) -> Result<impl IntoResponse, ServerError> {
    let p = land_dao::projects::get_by_name(name.clone(), Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let p = p.unwrap();
    land_dao::envs::update_envs(form, p.id, p.uuid).await?;
    Ok(response_redirect(
        format!("/projects/{}/settings", name).as_str(),
    ))
}
