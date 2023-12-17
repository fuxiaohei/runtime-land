use crate::{sign::SessionUser, PageVars, RenderEngine};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use serde::{Deserialize, Serialize};

/// index is the handler for /projects
pub async fn index(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
    }

    RenderHtml(
        "projects.hbs",
        engine,
        Vars {
            page: PageVars::new("Projects", "/projects"),
            user,
        },
    )
}
