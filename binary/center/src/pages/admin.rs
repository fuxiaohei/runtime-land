use super::auth::SessionUser;
use super::vars::{PageVars, PaginationVars, UserAdminVars, UserVars};
use super::AppEngine;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use axum_template::RenderHtml;
use land_dao::{deployment, project, user};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsQueryParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleProjectParams {
    pub csrf_token: String,
    pub uuid: String,
    pub owner_id: i32,
    pub action: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminUsersVars {
    pub page: PageVars,
    pub user: UserVars,
    pub user_count: u64,
    pub users: Vec<UserAdminVars>,
    pub pagination: PaginationVars,
}

pub async fn render_users(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<ProjectsQueryParams>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (users, pages, alls) = user::list_with_page(query.search.clone(), page, page_size)
        .await
        .unwrap();

    let user_ids: Vec<i32> = users.iter().map(|u| u.id).collect();
    let deploys_counts = deployment::list_counter_by_owners(user_ids.clone())
        .await
        .unwrap();
    let projects_counts = project::list_counter_by_owners(user_ids).await.unwrap();

    let users_vars = UserAdminVars::from_models(&users, projects_counts, deploys_counts)
        .await
        .unwrap();

    let page_vars = PageVars::new("Admin - Users".to_string(), "/admin/users".to_string());
    let user_vars = UserVars::new(&current_user);
    RenderHtml(
        "admin/users.hbs",
        engine,
        AdminUsersVars {
            page: page_vars,
            user: user_vars,
            user_count: alls,
            users: users_vars,
            pagination: PaginationVars::new(page, pages, "/admin/users"),
        },
    )
}
