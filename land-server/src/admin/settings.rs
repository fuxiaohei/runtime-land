use crate::{dash::ServerError, templates::Engine};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
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
    }
    Ok(RenderHtml(
        "admin/settings.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin Settings", BreadCrumbKey::AdminSettings, Some(user)),
        },
    ))
}
