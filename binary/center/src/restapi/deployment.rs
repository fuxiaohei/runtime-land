use super::{auth::CurrentUser, params, AppError};
use crate::settings;
use axum::{http::StatusCode, Extension, Json};
use tracing::{info, warn};
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
        String::from("todo"),
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

    let prod_domain = settings::DOMAIN.get().unwrap();
    let prod_protocol = settings::PROTOCOL.get().unwrap();

    // upload deploy chunk to storage
    let storage_path = format!("deployments/{}/{}.wasm", deployment.uuid, deployment.domain);
    let deployment_id = deployment.id;
    tokio::task::spawn(async move {
        match land_storage::write(&storage_path, payload.deploy_chunk).await {
            Ok(_) => {
                info!(
                    "success to upload deploy wasm to storage, storage_path:{}",
                    storage_path,
                );
            }
            Err(err) => {
                info!(
                    "failed to upload deploy wasm to storage, storage_path:{}, err:{}",
                    storage_path, err
                );
            }
        }

        // then update storage_path and deploy status
        match land_dao::deployment::set_storage_success(deployment_id, storage_path.clone()).await {
            Ok(_) => {}
            Err(err) => {
                warn!(
                    "failed to update deployment storage_path, id:{}, storage_path:{}, err:{}",
                    deployment_id, storage_path, err
                );
            }
        }
    });

    Ok((
        StatusCode::OK,
        Json(params::DeploymentResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            uuid: deployment.uuid,
            domain: deployment.domain.clone(),
            domain_url: format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain),
            prod_domain: String::new(),
            prod_url: String::new(),
            created_at: deployment.created_at.timestamp(),
            updated_at: deployment.updated_at.timestamp(),
            status: deployment.status,
            deploy_status: deployment.deploy_status,
        }),
    ))
}
