use crate::auth::CurrentUser;
use crate::{params, AppError};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use land_core::dao;
use tracing::{info, warn};
use validator::Validate;

/// lfetch_handler fetches a project by uuid for current user.
pub async fn fetch_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::FetchProjectRequest>,
) -> Result<(StatusCode, Json<params::ProjectData>), AppError> {
    payload.validate()?;
    let project = dao::project::find(current_user.id, payload.name.clone()).await?;
    if project.is_none() {
        warn!(
            "project not found, userid:{}, name:{}",
            current_user.id, payload.name
        );
        return Err(anyhow::anyhow!("project not found").into());
    }
    let project = project.unwrap();
    info!(
        "fetch_project success, userid:{}, name:{}",
        current_user.id, payload.name
    );
    Ok((
        StatusCode::OK,
        Json(params::ProjectData {
            name: project.name,
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
        }),
    ))
}

/// create_handler creates a new empty project for current user.
pub async fn create_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::FetchProjectRequest>,
) -> Result<(StatusCode, Json<params::ProjectData>), AppError> {
    payload.validate()?;
    let project =
        dao::project::create(payload.name.clone(), payload.language, current_user.id).await?;
    info!(
        "create_project success, userid:{}, name:{}, uuid: {}",
        current_user.id, payload.name, project.uuid
    );
    Ok((
        StatusCode::OK,
        Json(params::ProjectData {
            name: project.name,
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
        }),
    ))
}

/// list_handler lists all projects of current user.
pub async fn list_handler(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::ProjectData>>), AppError> {
    let projects = dao::project::list(current_user.id).await?;
    let values: Vec<params::ProjectData> = projects
        .into_iter()
        .map(|p| params::ProjectData {
            name: p.name,
            language: p.language,
            uuid: p.uuid,
            prod_deployment: p.prod_deploy_id,
            created_at: p.created_at.timestamp(),
            updated_at: p.updated_at.timestamp(),
        })
        .collect();
    Ok((StatusCode::OK, Json(values)))
}
