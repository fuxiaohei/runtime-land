use super::params::{CreateDeployRequest, DeploymentResponse, UpdateDeployRequest};
use super::SessionUser;
use crate::{apiv2::RouteError, settings};
use anyhow::Result;
use axum::{Extension, Json};
use hyper::StatusCode;
use land_dao::{deployment, project, Project};
use tracing::{debug_span, info, warn, Instrument};
use validator::Validate;

/// get_active_project gets the active project
async fn get_active_project(name: String, uuid: String, owner_id: i32) -> Result<Project> {
    let project = project::find_by_name(name, owner_id).await?;
    if project.is_none() {
        return Err(anyhow::anyhow!("project not found"));
    }
    let project = project.unwrap();
    if project.uuid != uuid {
        return Err(anyhow::anyhow!("project uuid not match"));
    }
    if project.status == project::Status::Active.to_string()
        || project.status == project::Status::Pending.to_string()
    {
        return Ok(project);
    }
    Err(anyhow::anyhow!("project is not active"))
}

/// upload_chunks uploads deploy chunks to storage
async fn upload_chunks(id: i32, storage_path: &str, deploy_chunk: Vec<u8>) -> anyhow::Result<()> {
    let upload_res = land_storage::write(storage_path, deploy_chunk).await;
    if upload_res.is_err() {
        deployment::set_deploy_status(id, deployment::DeployStatus::Failed).await?;
        return Err(upload_res.err().unwrap());
    }
    let _ = deployment::set_storage_success(id, storage_path.to_string()).await?;
    Ok(())
}

#[tracing::instrument(name = "[create_deployment_handler]", skip_all)]
pub async fn create_handler(
    Extension(current_user): Extension<SessionUser>,
    Json(payload): Json<CreateDeployRequest>,
) -> Result<(StatusCode, Json<DeploymentResponse>), RouteError> {
    payload.validate()?;

    let project =
        get_active_project(payload.project_name, payload.project_uuid, current_user.id).await?;

    let deployment = deployment::create(
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

    if project.status == project::Status::Pending.to_string() {
        let project = project::set_active(project.id).await?;
        info!("success to activate project from pending, project_name:{}, project_uuid:{}, project_status:{}", project.name, project.uuid, project.status);
    }

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

    let (prod_domain, prod_protocol) = settings::get_domains().await;

    Ok((
        StatusCode::OK,
        Json(DeploymentResponse::from_model(
            &deployment,
            &prod_domain,
            &prod_protocol,
        )),
    ))
}

#[tracing::instrument(name = "[update_deployment_handler]", skip_all)]
pub async fn update_handler(
    Extension(current_user): Extension<SessionUser>,
    Json(payload): Json<UpdateDeployRequest>,
) -> Result<(StatusCode, Json<DeploymentResponse>), RouteError> {
    match payload.action.as_str() {
        "publish" => {
            let deployment = deployment::publish(current_user.id, payload.deployment_uuid).await?;
            info!(
                "publish success, deployment_name:{}, deployment_uuid:{}",
                deployment.domain, deployment.uuid,
            );
            let (prod_domain, prod_protocol) = settings::get_domains().await;
            Ok((
                StatusCode::OK,
                Json(DeploymentResponse::from_model(
                    &deployment,
                    &prod_domain,
                    &prod_protocol,
                )),
            ))
        }
        "enable" => {
            let deployment = deployment::enable(current_user.id, payload.deployment_uuid).await?;
            info!(
                "enable success, deployment_name:{}, deployment_uuid:{}",
                deployment.domain, deployment.uuid,
            );
            let (prod_domain, prod_protocol) = settings::get_domains().await;
            Ok((
                StatusCode::OK,
                Json(DeploymentResponse::from_model(
                    &deployment,
                    &prod_domain,
                    &prod_protocol,
                )),
            ))
        }
        "disable" => {
            let deployment = deployment::disable(current_user.id, payload.deployment_uuid).await?;
            info!(
                "disable success, deployment_name:{}, deployment_uuid:{}",
                deployment.domain, deployment.uuid,
            );
            let (prod_domain, prod_protocol) = settings::get_domains().await;
            Ok((
                StatusCode::OK,
                Json(DeploymentResponse::from_model(
                    &deployment,
                    &prod_domain,
                    &prod_protocol,
                )),
            ))
        }
        _ => Err(RouteError(
            anyhow::anyhow!("action not supported"),
            StatusCode::BAD_REQUEST,
        )),
    }
}
