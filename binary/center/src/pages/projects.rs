use super::auth::SessionUser;
use super::vars::{PageVars, ProjectVars, UserVars};
use super::AppEngine;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use axum_template::RenderHtml;
use land_dao::{deployment, project};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
struct ProjectListVars {
    pub page: PageVars,
    pub user: UserVars,
    pub projects: Vec<Vec<ProjectVars>>,
    pub projects_count: u32,
}

pub async fn render(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new("Projects".to_string(), "/projects".to_string());
    let user_vars = UserVars::new(&current_user);
    let projects = project::list_available(current_user.id).await.unwrap();
    let counters = deployment::list_counter(current_user.id).await.unwrap();
    let projects_vars = ProjectVars::from_models(&projects, counters).await.unwrap();
    let mut result_vars = vec![];
    for project in &projects_vars {
        // if result_vars is empty, push a new vec
        if result_vars.is_empty() {
            result_vars.push(vec![project.clone()]);
            continue;
        };
        // if last element vec is not length 2, push a new vec
        if result_vars.last().unwrap().len() < 2 {
            result_vars.last_mut().unwrap().push(project.clone());
            continue;
        }
        // if last element vec is length 2, push a new vec
        result_vars.push(vec![project.clone()]);
    }
    let all_vars = ProjectListVars {
        page: page_vars,
        user: user_vars,
        projects: result_vars,
        projects_count: projects.len() as u32,
    };
    RenderHtml("projects.hbs", engine, all_vars)
}

pub async fn render_single(engine: AppEngine, Path(param): Path<String>) -> impl IntoResponse {
    debug!("param: {}", param);
    RenderHtml("project-single.hbs", engine, &())
}

pub async fn render_settings(engine: AppEngine, Path(param): Path<String>) -> impl IntoResponse {
    debug!("param: {}", param);
    RenderHtml("project-settings.hbs", engine, &())
}
