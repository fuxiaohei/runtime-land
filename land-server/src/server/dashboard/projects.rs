use crate::server::{
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars,
};
use axum::response::IntoResponse;

/// index is a handler for GET /
pub async fn index(engine: TemplateEngine) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
    }
    // redirect to /overview
    RenderHtmlMinified(
        "projects.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Projects", "projects"),
        },
    )
}
