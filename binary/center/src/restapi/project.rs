use super::{auth::CurrentUser, params, AppError};
use anyhow::Result;
use axum::{extract::Path, http::StatusCode, Extension, Json};
use tracing::info;
use validator::Validate;

#[tracing::instrument(name = "[project_create_handler]", skip_all)]
pub async fn create_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::CreateProjectRequest>,
) -> Result<(StatusCode, Json<params::ProjectResponse>), AppError> {
    payload.validate()?;

    let project = land_dao::project::create(
        payload.name,
        payload.prefix,
        payload.language,
        current_user.id,
    )
    .await?;
    info!(
        "success, project_name:{}, project_uuid:{}",
        project.name, project.uuid,
    );
    Ok((
        StatusCode::OK,
        Json(params::ProjectResponse {
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            prod_url: "".to_string(),
            status: project.status,
            name: project.name,
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
        }),
    ))
}

#[tracing::instrument(name = "[project_query_handler]", skip_all)]
pub async fn query_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<params::ProjectResponse>), AppError> {
    let project = land_dao::project::find_by_name(name, current_user.id).await?;
    if project.is_none() {
        return Err(AppError(
            anyhow::anyhow!("project not found"),
            StatusCode::NOT_FOUND,
        ));
    }
    let project = project.unwrap();
    Ok((
        StatusCode::OK,
        Json(params::ProjectResponse {
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            prod_url: "".to_string(),
            status: project.status,
            name: project.name,
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
        }),
    ))
}

/// get_active_project gets the active project
pub async fn get_active_project(
    name: String,
    uuid: String,
    owner_id: i32,
) -> Result<land_dao::Project> {
    let project = land_dao::project::find_by_name(name, owner_id).await?;
    if project.is_none() {
        return Err(anyhow::anyhow!("project not found"));
    }
    let project = project.unwrap();
    if project.uuid != uuid {
        return Err(anyhow::anyhow!("project uuid not match"));
    }
    if project.status == land_dao::project::Status::Active.to_string()
        || project.status == land_dao::project::Status::Pending.to_string()
    {
        return Ok(project);
    }
    Err(anyhow::anyhow!("project is not active"))
}
