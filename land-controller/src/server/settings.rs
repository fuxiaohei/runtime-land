use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form, Json};
use axum_csrf::CsrfToken;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::{response_redirect, ServerError};
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::PageVars;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct SettingsQuery {
    pub name: Option<String>,
    pub show: Option<String>,
}

/// settings is a handler for GET /settings
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: template::Engine,
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
    #[derive(Serialize)]
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
            "settings.hbs",
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

/// update is a handler for POST /settings
pub async fn update(
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

    Ok(response_redirect(
        format!("/admin/settings?show={}", f.name).as_str(),
    ))
}

#[derive(Deserialize, Debug)]
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
    if token.is_none() {
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
    Ok(response_redirect("/admin"))
}
