use crate::{dash::ServerError, templates::Engine};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use land_vars::{AuthUser, BreadCrumbKey, Page};
use serde::Serialize;

/// index is route of admin deploy logs page, /admin/deploy-logs/
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
        "admin/index.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin", BreadCrumbKey::AdminDeployLogs, Some(user)),
        },
    ))
}