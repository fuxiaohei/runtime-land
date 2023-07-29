use super::{auth::CurrentUser, params, AppError};
use axum::{http::StatusCode, Extension, Json};
use tracing::info;
use validator::Validate;

#[tracing::instrument(name = "[create_deployment]", skip_all)]
pub async fn create_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::CreateDeployRequest>,
) -> Result<(StatusCode, Json<params::DeploymentResponse>), AppError> {
    payload.validate()?;

    let project = super::project::get_active_project(
        payload.project_name,
        payload.project_uuid,
        current_user.id,
    )
    .await?;

    let deployment = land_dao::deployment::create(
        current_user.id,
        project.id,
        project.name,
        String::from("todo-storagepath"),
    )
    .await?;

    info!(
        "success, deployment_name:{}, deployment_uuid:{}",
        deployment.domain, deployment.uuid,
    );

    if project.status == land_dao::project::Status::Pending.to_string() {
        let project = land_dao::project::set_active(project.id).await?;
        info!("success to activate project from pending, project_name:{}, project_uuid:{}, project_status:{}", project.name, project.uuid, project.status);
    }

    Ok((
        StatusCode::OK,
        Json(params::DeploymentResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            uuid: deployment.uuid,
            domain: deployment.domain.clone(),
            domain_url: format!("http://{}.{}", deployment.domain, "local.dev"),
            prod_domain: deployment.prod_domain.clone(),
            prod_url: String::new(),
            created_at: deployment.created_at.timestamp(),
            updated_at: deployment.updated_at.timestamp(),
            status: deployment.status,
            deploy_status: deployment.deploy_status,
        }),
    ))
}
