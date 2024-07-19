use crate::{
    dash::{error_html, ServerError},
    templates::Engine,
};
use axum::{response::IntoResponse, Extension, Form};
use axum_htmx::HxRedirect;
use axum_template::RenderHtml;
use land_dao::{
    tokens::{self, Usage},
    workers,
};
use land_vars::{AuthUser, BreadCrumbKey, Page, Token, Worker};
use serde::{Deserialize, Serialize};
use tracing::info;

pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub tokens: Vec<Token>,
        pub token_create_url: String,
        pub token_remove_url: String,
        pub workers: Vec<Worker>,
    }
    let token_values = tokens::list(None, Some(tokens::Usage::Worker)).await?;
    let workers_value = workers::find_all(None).await?;
    let workers = workers_value.iter().map(Worker::new).collect();
    Ok(RenderHtml(
        "admin/workers.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Workers", BreadCrumbKey::AdminWorkers, Some(user)),
            tokens: Token::new_from_models(token_values),
            token_create_url: "/admin/workers/tokens/create".to_string(),
            token_remove_url: "/admin/workers/remove".to_string(),
            workers,
        },
    ))
}

/// TokenForm is the form for creating and removing a new token
#[derive(Deserialize, Debug)]
pub struct TokenForm {
    pub name: String,
    pub id: Option<i32>,
}

/// create_token create a new token for user
pub async fn create_token(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    let exist_token = tokens::get_by_name(&f.name, user.id, Some(Usage::Cmdline)).await?;
    if exist_token.is_some() {
        return Ok(error_html("Token name already exists").into_response());
    }
    let token = tokens::create(user.id, &f.name, 3600 * 24 * 365, Usage::Worker).await?;

    info!(
        owner_id = user.id,
        token_name = f.name,
        "Create new token: {:?}",
        token
    );

    let uri = axum::http::Uri::from_static("/admin/workers");
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}

/// remove_token removes a token by id
pub async fn remove_token(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    let token = tokens::get_by_name(&f.name, user.id, Some(Usage::Worker)).await?;
    if token.is_none() {
        return Ok(error_html("Token not found").into_response());
    }
    let token = token.unwrap();
    if token.id != f.id.unwrap_or(0) {
        return Ok(error_html("Token id not match").into_response());
    }
    tokens::set_expired(token.id, &f.name).await?;
    info!(
        owner_id = user.id,
        token_name = f.name,
        "Remove token: {}",
        token.id,
    );
    let uri = axum::http::Uri::from_static("/admin/workers");
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}
