use crate::{
    dash::ServerError,
    templates::{Engine, RenderHtmlMinified},
};
use axum::{response::IntoResponse, Extension};
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
    Ok(RenderHtmlMinified(
        "admin/index.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new(
                "Admin Deploy Logs",
                BreadCrumbKey::AdminDeployLogs,
                Some(user),
            ),
        },
    ))
}
