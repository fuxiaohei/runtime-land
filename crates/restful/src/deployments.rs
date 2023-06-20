use crate::auth::CurrentUser;
use crate::{params, AppError};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use land_core::storage::STORAGE;
use land_core::{dao, PROD_DOMAIN, PROD_PROTOCOL};
use tracing::{info, warn};
use validator::Validate;

/// create_handler creates a deployment for project.
pub async fn create_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::CreateDeployRequest>,
) -> Result<(StatusCode, Json<params::DeploymentData>), AppError> {
    payload.validate()?;

    let project = dao::project::find(current_user.id, payload.project_name.clone()).await?;
    if project.is_none() {
        warn!(
            "create deployment but project not found, userid:{}, name:{}",
            current_user.id, payload.project_name
        );
        return Err(anyhow::anyhow!("project not found").into());
    }
    let project = project.unwrap();
    if project.uuid != payload.project_uuid {
        warn!(
            "create deployment but project uuid not match, userid:{}, name:{}, uuid:{}, payload.uuid:{}",
            current_user.id, payload.project_name, project.uuid, payload.project_uuid
        );
        return Err(anyhow::anyhow!("project uuid not match").into());
    }

    // create deployment
    let deployment = dao::deployment::create(
        current_user.id,
        project.id,
        format!("{}-{}", project.name, payload.deploy_name),
        format!("fs://{}", payload.deploy_name),
    )
    .await?;

    // save wasm file
    let storage_path = format!("{}/{}.wasm", project.uuid, deployment.uuid);
    let storage = STORAGE.get().unwrap();
    storage.write(&storage_path, payload.deploy_chunk).await?;
    dao::deployment::update_storage(deployment.id, storage_path).await?;
    info!(
        "save deployment success, deploy_name:{}, deploy_uuid:{}",
        deployment.domain, deployment.uuid,
    );
    let prod_domain = PROD_DOMAIN.get().unwrap().clone();
    let prod_protocol = PROD_PROTOCOL.get().unwrap().clone();
    let resp = params::DeploymentData {
        id: deployment.id,
        project_id: project.id,
        domain: deployment.domain.clone(),
        domain_url: format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain),
        prod_domain: String::new(),
        prod_url: String::new(),
        uuid: deployment.uuid.clone(),
        deploy_status: deployment.deploy_status,
        prod_status: deployment.prod_status,
        created_at: deployment.created_at.timestamp(),
        updated_at: deployment.updated_at.timestamp(),
    };

    // deploy wasm in async task with deployment id
    // in future, deploy behavior should be a queue. It provides a better way to control.
    // Api need a method to get deployment status.
    let deploy_id = deployment.id;
    let deploy_uuid = deployment.uuid;
    tokio::spawn(async move {
        let res = land_core::region::local::deploy(deploy_id, deploy_uuid, false).await;
        if res.is_err() {
            warn!("deploy failed: {:?}", res.err().unwrap());
        }
    });

    Ok((StatusCode::OK, Json(resp)))
}

/// publish_handler publishes a deployment to production.
pub async fn publish_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::PublishDeployRequest>,
) -> Result<(StatusCode, Json<params::DeploymentData>), AppError> {
    payload.validate()?;

    let deployment =
        dao::deployment::publish(current_user.id, payload.deploy_id, payload.deploy_uuid).await?;
    let prod_domain = PROD_DOMAIN.get().unwrap().clone();
    let prod_protocol = PROD_PROTOCOL.get().unwrap().clone();

    let resp = params::DeploymentData {
        id: deployment.id,
        project_id: 0,
        domain: deployment.domain.clone(),
        domain_url: format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain),
        prod_domain: deployment.prod_domain.clone(),
        prod_url: format!(
            "{}://{}.{}",
            prod_protocol, deployment.prod_domain, prod_domain
        ),
        uuid: deployment.uuid.clone(),
        deploy_status: deployment.deploy_status,
        prod_status: deployment.prod_status,
        created_at: deployment.created_at.timestamp(),
        updated_at: deployment.updated_at.timestamp(),
    };

    let deploy_id = deployment.id;
    let deploy_uuid = deployment.uuid.clone();
    tokio::spawn(async move {
        let res = land_core::region::local::deploy(deploy_id, deploy_uuid, true).await;
        if res.is_err() {
            warn!("deploy failed: {:?}", res.err().unwrap());
        }
    });

    info!(
        "publish deployment success, deploy_name:{}, deploy_uuid:{}",
        deployment.domain, deployment.uuid
    );

    Ok((StatusCode::OK, Json(resp)))
}
