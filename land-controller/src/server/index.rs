use axum::response::IntoResponse;
use axum::Extension;
use axum_csrf::CsrfToken;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::ServerError;
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::PageVars;
use serde::Serialize;

/// index is a handler for GET /admin/
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: template::Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
    }

    let csrf = csrf_layer.authenticity_token()?;

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "index.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Dashboard", "admin-dashboard"),
                user,
                csrf,
            },
        ),
    )
        .into_response())
}
