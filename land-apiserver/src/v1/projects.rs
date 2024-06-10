use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use land_core_service::httputil::ServerJsonError;
use land_core_service::vars::{OkRespVar, PaginationVar, ProjectVar};
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

    let (projects, pagination) = land_dao::projects::list_paginate2(&q).await?;

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

#[derive(Debug, serde::Deserialize)]
pub struct SingleQuery {
    pub with_playground: Option<bool>,
}

pub async fn single(
    Extension(user): Extension<AuthUser>,
    Path(project_name): Path<String>,
    Query(q): Query<SingleQuery>,
) -> Result<impl IntoResponse, ServerJsonError> {
    debug!("get project single: {}, q:{:?}", project_name, q);
    let with_playground = q.with_playground.unwrap_or(false);
    if !with_playground {
        let p = land_dao::projects::get_by_name(project_name.clone(), Some(user.id)).await?;
        if p.is_none() {
            return Err(ServerJsonError(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("Project not found"),
            ));
        }
        let project_var = ProjectVar::new(&p.unwrap(), None).await?;
        return Ok(Json(project_var));
    }
    let (p, py) =
        land_dao::projects::get_project_by_name_with_playground(project_name, user.id).await?;
    let project_var = ProjectVar::new(&p, py.as_ref()).await?;
    Ok(Json(project_var))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateNamesForm {
    pub name: String,
    pub description: String,
}

/// update_names updates project names
pub async fn update_names(
    Extension(user): Extension<AuthUser>,
    Path(project_name): Path<String>,
    Json(f): Json<UpdateNamesForm>,
) -> Result<impl IntoResponse, ServerJsonError> {
    debug!(
        "update project names: {:?}, old_name: {:?}",
        f, project_name
    );
    let p = land_dao::projects::get_by_name(project_name, Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerJsonError(
            StatusCode::NOT_FOUND,
            anyhow::anyhow!("Project not found"),
        ));
    }
    let p = p.unwrap();
    land_dao::projects::update_name(p.id, f.name, f.description).await?;
    Ok(Json(OkRespVar::new()))
}

/// delete deletes a project
pub async fn delete(
    Extension(user): Extension<AuthUser>,
    Path(project_name): Path<String>,
) -> Result<impl IntoResponse, ServerJsonError> {
    debug!("delete project: {}", project_name);
    let p = land_dao::projects::get_by_name(project_name, Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerJsonError(
            StatusCode::NOT_FOUND,
            anyhow::anyhow!("Project not found"),
        ));
    }
    let p = p.unwrap();
    land_dao::projects::delete(p.id, p.name).await?;
    Ok(Json(OkRespVar::new()))
}
