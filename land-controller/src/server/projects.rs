use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use axum_csrf::CsrfToken;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::ServerError;
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::{PageVars, PaginationVar, ProjectVar};
use land_dao::user::UserStatus;
use tracing::info;

#[derive(serde::Deserialize, Debug)]
pub struct ProjectsQuery {
    page: Option<u64>,
    size: Option<u64>,
}

/// index is a handler for GET /projects/
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: template::Engine,
    Query(q): Query<ProjectsQuery>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
        projects: Vec<ProjectVar>,
        pagination: PaginationVar,
    }

    let csrf = csrf_layer.authenticity_token()?;
    let page = q.page.unwrap_or(1);
    let page_size = q.size.unwrap_or(10);
    let (project_values, pages) = land_dao::projects::list_paginate(page, page_size).await?;
    let mut projects = ProjectVar::from_models_vec(project_values).await?;

    // collect projects user id, need unique
    let user_ids: Vec<i32> = projects.iter().map(|p| p.user_id).collect();
    let user_ids = user_ids
        .into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let users = land_dao::user::list_infos(user_ids).await?;

    // fill user email and nickname
    for p in projects.iter_mut() {
        let user = users.get(&p.user_id);
        if let Some(user) = user {
            p.user_email.clone_from(&user.email);
            p.user_nickname.clone_from(&user.nick_name);
        }
    }

    // fill pagination
    let pagination = PaginationVar::new(
        page,
        page_size,
        pages.number_of_items,
        pages.number_of_pages,
        "/projects",
    );

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "projects.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Projects", "admin-projects"),
                user,
                csrf,
                projects,
                pagination,
            },
        ),
    )
        .into_response())
}

#[derive(serde::Deserialize, Debug)]
pub struct ProjectRedeployQuery {
    project_id: i32,
}

pub async fn redeploy(
    Form(f): Form<ProjectRedeployQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let project = land_dao::projects::get_by_id(f.project_id, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project = project.unwrap();

    let user = land_dao::user::get_info_by_id(project.user_id, Some(UserStatus::Active)).await?;
    if user.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "User not found",
        ));
    }
    let user = user.unwrap();
    let dp = land_dao::deployment::create(
        user.id,
        user.uuid,
        project.id,
        project.uuid,
        project.prod_domain,
    )
    .await?;
    info!("Redeploy project: {}, dp: {}", project.id, dp.id);
    Ok(())
}

#[derive(serde::Deserialize, Debug)]
pub struct ProjectDisableQuery {
    project_id: i32,
}

/// disable is a handler for POST /projects/disable
pub async fn disable(
    Form(f): Form<ProjectDisableQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let project = land_dao::projects::get_by_id(f.project_id, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project = project.unwrap();
    land_dao::projects::set_disabled(project.id).await?;
    info!(
        "Disable project: {}, name: {}",
        project.id, project.prod_domain
    );
    Ok(())
}

/// ProjectEnableQuery is a queyr struct for enable_project
type ProjectEnableQuery = ProjectDisableQuery;

/// enable_project is a handler for POST /projects/enable
pub async fn enable(
    Form(f): Form<ProjectEnableQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let project = land_dao::projects::get_by_id(f.project_id, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project = project.unwrap();
    land_dao::projects::set_enabled(project.id).await?;
    info!(
        "Enable project: {}, name: {}",
        project.id, project.prod_domain
    );
    Ok(())
}
