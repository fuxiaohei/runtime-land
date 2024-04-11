use super::auth::SessionUser;
use crate::server::{
    redirect_response,
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars, ServerError,
};
use axum::{response::IntoResponse, Extension, Form};
use axum_csrf::CsrfToken;
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
    }
    let csrf = csrf_layer.authenticity_token()?;
    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "settings.hbs",
            engine,
            IndexVars {
                page: PageVars::new("Account Settings", "settings"),
                user,
                csrf,
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
