use crate::auth::CurrentUser;
use crate::{params, AppError};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use land_core::dao;
use tracing::info;
use validator::Validate;

/// list_handler lists all tokens of current user.
pub async fn fetch_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::FetchProjectRequest>,
) -> Result<(StatusCode, Json<params::ProjectData>), AppError> {
    payload.validate()?;
    let project = dao::project::find(current_user.id, payload.name.clone()).await?;
    if project.is_none() {
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
