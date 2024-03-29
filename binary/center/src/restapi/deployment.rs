use super::{auth::CurrentUser, params, AppError};
use crate::{region::conf, settings};
use axum::{extract::Path, http::StatusCode, Extension, Json};
use tracing::{debug_span, info, warn, Instrument};
use validator::Validate;

/// upload_chunks uploads deploy chunks to storage
async fn upload_chunks(id: i32, storage_path: &str, deploy_chunk: Vec<u8>) -> anyhow::Result<()> {
    let upload_res = land_storage::write(storage_path, deploy_chunk).await;
    if upload_res.is_err() {
        land_dao::deployment::set_deploy_status(id, land_dao::deployment::DeployStatus::Failed)
            .await?;
        return Err(upload_res.err().unwrap());
    }
    let _ = land_dao::deployment::set_storage_success(id, storage_path.to_string()).await?;
    // trigger build conf
    conf::trigger().await;
    Ok(())
}

#[tracing::instrument(name = "[create_dp]", skip_all)]
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
        payload.deploy_chunk.len() as i32,
        payload.deploy_content_type,
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

    let (prod_domain, prod_protocol) = settings::get_domains().await;

    // upload deploy chunk to storage
    let storage_path = format!("deployments/{}/{}.wasm", project.uuid, deployment.domain);
    let deployment_id = deployment.id;
    tokio::task::spawn(
        async move {
            match upload_chunks(deployment_id, &storage_path, payload.deploy_chunk).await{
                Ok(_) => info!("success to upload deploy chunk to storage, deployment_id:{}, storage_path:{}", deployment_id, storage_path),
                Err(e) => warn!("failed to upload deploy chunk to storage, deployment_id:{}, storage_path:{}, err:{}", deployment_id, storage_path, e),
            }
        }
        .instrument(debug_span!("[upload_deploy_chunk]")),
    );

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

/// publish_handler publishes the deployment
#[tracing::instrument(name = "[publish_dp]", skip_all)]
pub async fn publish_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(uuid): Path<String>,
) -> Result<(StatusCode, Json<params::DeploymentResponse>), AppError> {
    let deployment = land_dao::deployment::publish(current_user.id, uuid).await?;
    info!(
        "success, deployment_name:{}, deployment_uuid:{}",
        deployment.domain, deployment.uuid,
    );

    conf::trigger().await;

    let (prod_domain, prod_protocol) = settings::get_domains().await;

    Ok((
        StatusCode::OK,
        Json(params::DeploymentResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            uuid: deployment.uuid,
            domain: deployment.domain.clone(),
            domain_url: format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain),
            prod_domain: deployment.prod_domain.clone(),
            prod_url: format!(
                "{}://{}.{}",
                prod_protocol, deployment.prod_domain, prod_domain
            ),
            created_at: deployment.created_at.timestamp(),
            updated_at: deployment.updated_at.timestamp(),
            status: deployment.status,
            deploy_status: deployment.deploy_status,
        }),
    ))
}

/// disable_handler disables a deployment
#[tracing::instrument(name = "[disable_dp]", skip_all)]
pub async fn disable_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(uuid): Path<String>,
) -> Result<StatusCode, AppError> {
    let deployment = land_dao::deployment::disable(current_user.id, uuid).await?;
    info!(
        "success, deployment_name:{}, deployment_uuid:{}",
        deployment.domain, deployment.uuid,
    );
    conf::trigger().await;
    Ok(StatusCode::OK)
}

/// enable_handler enables a deployment
#[tracing::instrument(name = "[enable_ep]", skip_all)]
pub async fn enable_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(uuid): Path<String>,
) -> Result<StatusCode, AppError> {
    let deployment = land_dao::deployment::enable(current_user.id, uuid).await?;
    info!(
        "success, deployment_name:{}, deployment_uuid:{}",
        deployment.domain, deployment.uuid,
    );
    conf::trigger().await;
    Ok(StatusCode::OK)
}
