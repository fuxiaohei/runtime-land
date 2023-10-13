use super::params::{CreateProjectRequest, ProjectOverview, ProjectRenameRequest, ProjectResponse};
use super::SessionUser;
use crate::{apiv2::RouteError, settings};
use axum::{extract::Path, Extension, Json};
use hyper::StatusCode;
use land_dao::{deployment, project};
use tracing::info;
use validator::Validate;

#[tracing::instrument(name = "[project_list_handler]", skip_all)]
pub async fn list_handler(
    Extension(user): Extension<SessionUser>,
) -> Result<(StatusCode, Json<Vec<ProjectOverview>>), RouteError> {
    let projects = project::list_available(user.id).await?;
    let counters = deployment::list_counter(user.id).await?;
    let overviews = ProjectOverview::from_vec(projects, counters, user.id).await?;
    Ok((StatusCode::OK, Json(overviews)))
}

#[tracing::instrument(name = "[project_create_handler]", skip_all)]
pub async fn create_handler(
    Extension(current_user): Extension<SessionUser>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), RouteError> {
    payload.validate()?;
    let project = project::create(
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

    // if template is set, generate default production deployment with template wasm file
    if let Some(template) = payload.template {
        let wasm_file = format!("templates-wasm/{}.component.wasm", template.name);
        let chunk = std::fs::read(wasm_file)?;
        let deployment = super::deployment::create_deployment(
            current_user.id,
            &project,
            chunk,
            "application/wasm".to_string(),
        )
        .await?;
        info!(
            "success with template, deployment_name:{}, deployment_uuid:{}, template:{}",
            deployment.domain, deployment.uuid, template.name,
        );

        // publish deployment
        let deployment = deployment::publish(current_user.id, deployment.uuid).await?;
        info!(
            "publish success, deployment_name:{}, deployment_uuid:{}",
            deployment.domain, deployment.uuid,
        );
    }

    let (prod_domain, _) = settings::get_domains().await;
    let project_response = ProjectResponse::from_model(&project, &prod_domain);
    Ok((StatusCode::OK, Json(project_response)))
}

#[tracing::instrument(name = "[project_overview_handler]", skip_all)]
pub async fn overview_handler(
    Extension(current_user): Extension<SessionUser>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ProjectOverview>), RouteError> {
    let project = land_dao::project::find_by_name(name, current_user.id).await?;
    if project.is_none() {
        return Err(RouteError(
            anyhow::anyhow!("project not found"),
            StatusCode::NOT_FOUND,
        ));
    }
    let project = project.unwrap();
    let overview = ProjectOverview::from_model(&project).await?;
    Ok((StatusCode::OK, Json(overview)))
}

#[tracing::instrument(name = "[project_rename_handler]", skip_all)]
pub async fn rename_handler(
    Extension(current_user): Extension<SessionUser>,
    Json(payload): Json<ProjectRenameRequest>,
) -> Result<StatusCode, RouteError> {
    let project = project::rename(
        current_user.id,
        payload.old_name.clone(),
        payload.new_name.clone(),
    )
    .await?;

    if project.prod_deploy_id > 0 {
        deployment::update_prod_domain(project.prod_deploy_id, project.name).await?;
    }
    info!(
        "success, owner_id:{}, old_name:{}, new_name:{}",
        current_user.id, payload.old_name, payload.new_name
    );

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "[project_remove_handler]", skip_all)]
pub async fn remove_handler(
    Extension(current_user): Extension<SessionUser>,
    Path(name): Path<String>,
) -> Result<StatusCode, RouteError> {
    // name is uuid in this api
    let project_id = project::remove_project(current_user.id, name.clone()).await?;
    info!("success, owner_id:{}, uuid:{}", current_user.id, name);
    // set related deployments to deleted
    deployment::set_deleted_by_project(project_id).await?;
    Ok(StatusCode::OK)
}
