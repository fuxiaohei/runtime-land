use super::auth::SessionUser;
use super::vars::{PageVars, PaginationVars, UserVars};
use super::AppEngine;
use crate::pages::vars::ProjectAdminVars;
use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Redirect};
use axum::Extension;
use axum_template::RenderHtml;
use hyper::StatusCode;
use land_dao::{deployment, project, user};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
struct AdminProjectVars {
    pub page: PageVars,
    pub user: UserVars,
    pub project_count: u64,
    pub projects: Vec<ProjectAdminVars>,
    pub pagination: PaginationVars,
    pub search: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsQueryParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub search: Option<String>,
}

pub async fn render_projects(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<ProjectsQueryParams>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (projects, pages, alls) =
        project::list_all_available_with_page(query.search.clone(), page, page_size)
            .await
            .unwrap();

    let project_ids: HashSet<i32> = projects.iter().map(|p| p.id).collect();
    let deploy_counts = deployment::list_counter_by_projects(project_ids.into_iter().collect())
        .await
        .unwrap();

    let owner_ids: HashSet<i32> = projects.iter().map(|p| p.owner_id).collect();
    let owners = user::list_by_ids(owner_ids.into_iter().collect())
        .await
        .unwrap();

    let page_vars = PageVars::new(
        "Admin - Projects".to_string(),
        "/admin/projects".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    let project_vars = ProjectAdminVars::from_models(&projects, deploy_counts, owners)
        .await
        .unwrap();
    let pagination_vars = PaginationVars::new(page, pages, "/admin/projects");

    RenderHtml(
        "admin/projects.hbs",
        engine,
        AdminProjectVars {
            page: page_vars,
            user: user_vars,
            project_count: alls,
            projects: project_vars,
            pagination: pagination_vars,
            search: query.search.unwrap_or_default(),
        },
    )
}

pub async fn handle_project_disable(Path(uuid): Path<String>) -> Result<Redirect, StatusCode> {
    let project = match project::find_by_uuid(uuid).await {
        Ok(p) => {
            if p.is_none() {
                return Err(StatusCode::NOT_FOUND);
            }
            p.unwrap()
        }
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    project::set_inactive(project.id).await.unwrap();
    Ok(Redirect::to("/admin/projects"))
}

pub async fn handle_project_enable(Path(uuid): Path<String>) -> Result<Redirect, StatusCode> {
    let project = match project::find_by_uuid(uuid).await {
        Ok(p) => {
            if p.is_none() {
                return Err(StatusCode::NOT_FOUND);
            }
            p.unwrap()
        }
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    project::set_active(project.id).await.unwrap();
    Ok(Redirect::to("/admin/projects"))
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminDeploymentsVars {
    pub page: PageVars,
    pub user: UserVars,
}

pub async fn render_deployments(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new(
        "Admin - Deployments".to_string(),
        "/admin/deployments".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    RenderHtml(
        "admin/deployments.hbs",
        engine,
        AdminDeploymentsVars {
            page: page_vars,
            user: user_vars,
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminUsersVars {
    pub page: PageVars,
    pub user: UserVars,
}

pub async fn render_users(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new("Admin - Users".to_string(), "/admin/users".to_string());
    let user_vars = UserVars::new(&current_user);
    RenderHtml(
        "admin/users.hbs",
        engine,
        AdminUsersVars {
            page: page_vars,
            user: user_vars,
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminEndpointsVars {
    pub page: PageVars,
    pub user: UserVars,
}

pub async fn render_endpoints(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new(
        "Admin - Endpoints".to_string(),
        "/admin/endpoints".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    RenderHtml(
        "admin/endpoints.hbs",
        engine,
        AdminEndpointsVars {
            page: page_vars,
            user: user_vars,
        },
    )
}
