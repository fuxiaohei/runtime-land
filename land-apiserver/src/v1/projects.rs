use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use land_core_service::httputil::ServerJsonError;
use land_core_service::vars::{PaginationVar, ProjectVar};
use land_dao::projects;
use land_service::clerk::AuthUser;
use tracing::debug;

#[derive(Debug, serde::Serialize)]
pub struct ListResp {
    page: PaginationVar,
    projects: Vec<ProjectVar>,
}

pub async fn list(
    Extension(user): Extension<AuthUser>,
    Query(mut q): Query<land_dao::projects::ListQuery>,
) -> Result<impl IntoResponse, ServerJsonError> {
    q.user_id = Some(user.id);
    debug!("list projects: {:?}", q);

    let (projects, pagination) = projects::list_paginate2(&q).await?;

    let projects_vars = ProjectVar::from_models_vec(projects).await?;
    let page_vars = PaginationVar::new(
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(10),
        pagination.number_of_items,
        pagination.number_of_pages,
        "",
    );

    Ok(Json(ListResp {
        page: page_vars,
        projects: projects_vars,
    }))
}
