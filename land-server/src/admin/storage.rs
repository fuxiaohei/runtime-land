use crate::{
    dash::{ok_html, ServerError},
    templates::Engine,
};
use axum::{response::IntoResponse, Extension, Form};
use axum_template::RenderHtml;
use land_core::storage;
use land_vars::{AuthUser, BreadCrumbKey, Page};
use serde::Serialize;

pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub storage: storage::Vars,
    }
    Ok(RenderHtml(
        "admin/storage.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Storage", BreadCrumbKey::AdminStorage, Some(user)),
            storage: storage::Vars::get().await?,
        },
    ))
}

/// update_storage for admin storage, POST /admin/storage
pub async fn update(Form(form): Form<storage::Form>) -> Result<impl IntoResponse, ServerError> {
    storage::update_by_form(form).await?;
    Ok(ok_html("Storage updated"))
}
