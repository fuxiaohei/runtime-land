use super::{
    auth::CurrentUser,
    params::{self, ProjectResponse},
    AppError,
};
use crate::settings;
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

#[tracing::instrument(name = "[project_list_handler]", skip_all)]
pub async fn list_handler(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::ProjectOverview>>), AppError> {
    let projects = land_dao::project::list_available(current_user.id).await?;
    let counters = land_dao::deployment::list_counter(current_user.id).await?;

    let prod_domain = settings::DOMAIN.get().unwrap();
    let prod_protocol = settings::PROTOCOL.get().unwrap();

    let mut project_overviews = Vec::new();
    for project in projects {
        let counter = counters.get(&project.id).unwrap_or(&0);

        let project_response = ProjectResponse {
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            prod_url: "".to_string(),
            status: project.status,
            name: project.name,
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
        };

        let mut overview = params::ProjectOverview {
            deployments_count: *counter,
            deployments: None,
            prod_deployment: None,
            project: project_response,
        };

        // load prod deployment
        if project.prod_deploy_id > 0 {
            let deployment =
                land_dao::deployment::find_by_id(current_user.id, project.prod_deploy_id).await?;
            if deployment.is_some() {
                let deployment = deployment.unwrap();
                overview.project.prod_url =
                    format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain);
                overview.prod_deployment = Some(params::DeploymentResponse {
                    id: deployment.id,
                    project_id: deployment.project_id,
                    uuid: deployment.uuid,
                    domain: deployment.domain.clone(),
                    domain_url: format!(
                        "{}://{}.{}",
                        prod_protocol, deployment.domain, prod_domain
                    ),
                    prod_domain: deployment.prod_domain.clone(),
                    prod_url: format!(
                        "{}://{}.{}",
                        prod_protocol, deployment.prod_domain, prod_domain
                    ),
                    created_at: deployment.created_at.timestamp(),
                    updated_at: deployment.updated_at.timestamp(),
                    status: deployment.status,
                    deploy_status: deployment.deploy_status,
                });
            }
        }

        project_overviews.push(overview);
    }

    info!(
        "success, owner_id:{}, count:{}",
        current_user.id,
        project_overviews.len()
    );

    Ok((StatusCode::OK, Json(project_overviews)))
}

#[tracing::instrument(name = "[project_remove_handler]", skip_all)]
pub async fn remove_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    // name is uuid in this api
    land_dao::project::remove_project(current_user.id, name.clone()).await?;
    info!("success, owner_id:{}, uuid:{}", current_user.id, name);
    Ok(StatusCode::OK)
}
