use super::auth::SessionUser;
use super::vars::{PageVars, UserVars};
use super::AppEngine;
use axum::response::IntoResponse;
use axum::Extension;
use axum_template::RenderHtml;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AdminProjectVars {
    pub page: PageVars,
    pub user: UserVars,
}

pub async fn render_projects(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new(
        "Admin - Projects".to_string(),
        "/admin/projects".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    RenderHtml(
        "admin/projects.hbs",
        engine,
        AdminProjectVars {
            page: page_vars,
            user: user_vars,
        },
    )
}
