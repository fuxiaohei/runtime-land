use crate::auth::CurrentUser;
use crate::{params, AppError};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{Form, Json};
use land_core::{dao, PROD_DOMAIN, PROD_PROTOCOL};
use tracing::{info, warn};
use validator::Validate;

/// lfetch_handler fetches a project by uuid for current user.
pub async fn fetch_handler(
    Extension(current_user): Extension<CurrentUser>,
    Form(payload): Form<params::FetchProjectRequest>,
) -> Result<(StatusCode, Json<params::ProjectData>), AppError> {
    payload.validate()?;
    let project = dao::project::find(current_user.id, payload.name.clone()).await?;
    if project.is_none() {
        warn!(
            "project not found, userid:{}, name:{}",
            current_user.id, payload.name
        );
        return Err(AppError(
            anyhow::anyhow!("project not found"),
            StatusCode::NOT_FOUND,
        ));
    }
    let project = project.unwrap();
    info!(
        "fetch_project success, userid:{}, name:{}",
        current_user.id, payload.name
    );
    let prod_url = if project.prod_deploy_id > 0 {
        format!(
            "{}://{}.{}",
            PROD_PROTOCOL.get().unwrap(),
            project.name.clone(),
            PROD_DOMAIN.get().unwrap()
        )
    } else {
        String::new()
    };
    Ok((
        StatusCode::OK,
        Json(params::ProjectData {
            name: project.name,
            language: project.language,
            uuid: project.uuid,
            prod_deployment: project.prod_deploy_id,
            created_at: project.created_at.timestamp(),
            updated_at: project.updated_at.timestamp(),
            prod_url,
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
            prod_url: String::new(),
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
        .map(|p| {
            let prod_url = if p.prod_deploy_id > 0 {
                format!(
                    "{}://{}.{}",
                    PROD_PROTOCOL.get().unwrap(),
                    p.name.clone(),
                    PROD_DOMAIN.get().unwrap()
                )
            } else {
                String::new()
            };
            params::ProjectData {
                name: p.name,
                language: p.language,
                uuid: p.uuid,
                prod_deployment: p.prod_deploy_id,
                created_at: p.created_at.timestamp(),
                updated_at: p.updated_at.timestamp(),
                prod_url,
            }
        })
        .collect();
    info!(
        "list_projects success, userid:{}, count:{}",
        current_user.id,
        values.len()
    );
    Ok((StatusCode::OK, Json(values)))
}

/// overview_handler returns a project overview.
pub async fn overview_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::FetchProjectRequest>,
) -> Result<(StatusCode, Json<params::ProjectOverview>), AppError> {
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
    let prod_domain = PROD_DOMAIN.get().unwrap().clone();
    let prod_protocol = PROD_PROTOCOL.get().unwrap().clone();
    let mut resp = params::ProjectOverview {
        id: project.id,
        name: project.name.clone(),
        uuid: project.uuid,
        prod_deployment_id: project.prod_deploy_id,
        created_at: project.created_at.timestamp(),
        updated_at: project.updated_at.timestamp(),
        deployments: vec![],
        prod_deployment: None,
        prod_url: format!(
            "{}://{}.{}",
            prod_protocol,
            project.name.clone(),
            prod_domain
        ),
    };

    // if prod_deployment is set, load deployment data
    if resp.prod_deployment_id > 0 {
        let deployment = dao::deployment::find_by_id(resp.prod_deployment_id).await?;
        if deployment.is_some() {
            let deployment = deployment.unwrap();
            resp.prod_deployment = Some(params::DeploymentData {
                id: deployment.id,
                domain: deployment.domain.clone(),
                domain_url: format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain),
                prod_domain: deployment.prod_domain.clone(),
                prod_url: format!(
                    "{}://{}.{}",
                    prod_protocol, deployment.prod_domain, prod_domain
                ),
                prod_status: deployment.prod_status,
                deploy_status: deployment.deploy_status,
                uuid: deployment.uuid,
                project_id: deployment.project_id,
                created_at: deployment.created_at.timestamp(),
                updated_at: deployment.updated_at.timestamp(),
            });
        }
    }

    // load project deployments
    let deployments = dao::deployment::list(project.owner_id, project.id, 10).await?;
    let values: Vec<params::DeploymentData> = deployments
        .into_iter()
        .map(|d| params::DeploymentData {
            id: d.id,
            domain: d.domain.clone(),
            domain_url: format!("{}://{}.{}", prod_protocol, d.domain, prod_domain),
            prod_domain: String::new(),
            prod_url: String::new(),
            prod_status: d.prod_status,
            deploy_status: d.deploy_status,
            uuid: d.uuid,
            project_id: d.project_id,
            created_at: d.created_at.timestamp(),
            updated_at: d.updated_at.timestamp(),
        })
        .collect();
    resp.deployments = values;
    info!(
        "project_overview success, userid:{}, name:{}, prod:{}, deployments:{}",
        current_user.id,
        payload.name,
        resp.prod_deployment_id,
        resp.deployments.len(),
    );
    Ok((StatusCode::OK, Json(resp)))
}
