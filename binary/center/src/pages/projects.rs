use super::auth::SessionUser;
use super::vars::{DeploymentVars, PageVars, ProjectVars, UserVars};
use super::AppEngine;
use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Form};
use axum_template::RenderHtml;
use land_dao::{deployment, project};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};

#[derive(Debug, Serialize, Deserialize)]
struct ProjectListVars {
    pub page: PageVars,
    pub user: UserVars,
    pub projects: Vec<Vec<ProjectVars>>,
    pub projects_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct ProjectListSearchParams {
    search: Option<String>,
}

pub async fn render(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    Query(search_params): Query<ProjectListSearchParams>,
) -> impl IntoResponse {
    let page_vars = PageVars::new("Projects".to_string(), "/projects".to_string());
    let user_vars = UserVars::new(&current_user);
    let projects = project::list_available(current_user.id).await.unwrap();
    let counters = deployment::list_counter(current_user.id).await.unwrap();
    let projects_vars = ProjectVars::from_models(&projects, counters).await.unwrap();
    let mut result_vars = vec![];
    let search_filter = search_params.search.unwrap_or("".to_string());
    let mut counts: u32 = 0;
    for project in &projects_vars {
        // filter by search
        if !search_filter.is_empty() && !project.name.contains(&search_filter) {
            continue;
        }
        counts += 1;
        // if result_vars is empty, push a new vec
        if result_vars.is_empty() {
            result_vars.push(vec![project.clone()]);
            continue;
        };
        // if last element vec is not length 2, push a new vec
        if result_vars.last().unwrap().len() < 2 {
            result_vars.last_mut().unwrap().push(project.clone());
            continue;
        }
        // if last element vec is length 2, push a new vec
        result_vars.push(vec![project.clone()]);
    }
    let all_vars = ProjectListVars {
        page: page_vars,
        user: user_vars,
        projects: result_vars,
        projects_count: counts,
    };
    RenderHtml("projects.hbs", engine, all_vars)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProjectSingleVars {
    pub page: PageVars,
    pub user: UserVars,
    pub project: ProjectVars,
    pub deployments: Vec<DeploymentVars>,
}

fn project_notfound_vars(project_name: String, user: &SessionUser) -> ProjectSingleVars {
    let page_vars = PageVars::new(project_name.clone(), format!("/projects/{}", project_name));
    let user_vars = UserVars::new(user);
    ProjectSingleVars {
        page: page_vars,
        user: user_vars,
        project: ProjectVars::default(),
        deployments: vec![],
    }
}

pub async fn render_single(
    engine: AppEngine,
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let project_name = param.clone();
    let project = project::find_by_name(project_name.clone(), current_user.id)
        .await
        .unwrap();
    if project.is_none() {
        return RenderHtml(
            "project-notfound.hbs",
            engine,
            project_notfound_vars(project_name, &current_user),
        );
    }
    let project = project.unwrap();
    let deployments = deployment::list_by_project_id(project.id).await.unwrap();
    let page_vars = PageVars::new(project_name.clone(), format!("/projects/{}", project_name));
    let user_vars = UserVars::new(&current_user);
    let project_var = ProjectVars::from_model(&project).await.unwrap();
    let deployments_vars = DeploymentVars::from_models(&deployments).await.unwrap();
    let vars = ProjectSingleVars {
        page: page_vars,
        user: user_vars,
        project: project_var,
        deployments: deployments_vars,
    };
    RenderHtml("project-single.hbs", engine, vars)
}

pub async fn render_settings(
    engine: AppEngine,
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let project_name = param.clone();
    let project = project::find_by_name(project_name.clone(), current_user.id)
        .await
        .unwrap();
    if project.is_none() {
        return RenderHtml(
            "project-notfound.hbs",
            engine,
            project_notfound_vars(project_name, &current_user),
        );
    }
    let project = project.unwrap();
    let page_vars = PageVars::new(project_name.clone(), format!("/projects/{}", project_name));
    let user_vars = UserVars::new(&current_user);
    let project_var = ProjectVars::from_model(&project).await.unwrap();
    let vars = ProjectSingleVars {
        page: page_vars,
        user: user_vars,
        project: project_var,
        deployments: vec![],
    };
    RenderHtml("project-settings.hbs", engine, vars)
}

#[derive(Debug, Deserialize)]
pub struct DeploymentHandleParams {
    uuid: String,
}

async fn parse_deployments(
    param: String,
    owner_id: i32,
    deployment_uuid: String,
) -> anyhow::Result<land_dao::Deployment> {
    let project = project::find_by_name(param.clone(), owner_id)
        .await
        .unwrap();
    if project.is_none() {
        return Err(anyhow::anyhow!("project not found, name: {}", param));
    }
    let project = project.unwrap();

    let deployment = deployment::find_by_uuid(owner_id, deployment_uuid.clone())
        .await
        .unwrap();
    if deployment.is_none() {
        return Err(anyhow::anyhow!(
            "deployment not found, uuid: {}",
            deployment_uuid
        ));
    }
    let deployment = deployment.unwrap();
    if deployment.project_id != project.id {
        error!(
            "deployment not match project, project_id: {}, deployment_id: {}",
            project.id, deployment.id
        );
        return Err(anyhow::anyhow!(
            "deployment not match project, project_id: {}, deployment_id: {}",
            project.id,
            deployment.id
        ));
    }
    Ok(deployment)
}

#[instrument(skip_all, fields(uuid = %query.uuid),"deployment_publish")]
pub async fn handle_publish(
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<DeploymentHandleParams>,
) -> Redirect {
    let deployment =
        match parse_deployments(param.clone(), current_user.id, query.uuid.clone()).await {
            Ok(v) => v,
            Err(e) => {
                error!("parse_deployments error: {}", e);
                return Redirect::to(format!("/projects/{}", param).as_str());
            }
        };
    deployment::publish(deployment.owner_id, deployment.uuid)
        .await
        .unwrap();
    info!(
        "deployment published, uuid: {}, domain: {}",
        query.uuid, deployment.domain
    );
    Redirect::to(format!("/projects/{}", param).as_str())
}

#[instrument(skip_all, fields(uuid = %query.uuid),"deployment_enable")]
pub async fn handle_enable(
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<DeploymentHandleParams>,
) -> Redirect {
    let deployment =
        match parse_deployments(param.clone(), current_user.id, query.uuid.clone()).await {
            Ok(v) => v,
            Err(e) => {
                error!("parse_deployments error: {}", e);
                return Redirect::to(format!("/projects/{}", param).as_str());
            }
        };
    deployment::enable(deployment.owner_id, deployment.uuid)
        .await
        .unwrap();
    info!(
        "deployment enabled, uuid: {}, domain: {}",
        query.uuid, deployment.domain
    );
    Redirect::to(format!("/projects/{}", param).as_str())
}

#[instrument(skip_all, fields(uuid = %query.uuid),"deployment_disable")]
pub async fn handle_disable(
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<DeploymentHandleParams>,
) -> Redirect {
    let deployment =
        match parse_deployments(param.clone(), current_user.id, query.uuid.clone()).await {
            Ok(v) => v,
            Err(e) => {
                error!("parse_deployments error: {}", e);
                return Redirect::to(format!("/projects/{}", param).as_str());
            }
        };
    deployment::disable(deployment.owner_id, deployment.uuid)
        .await
        .unwrap();
    info!(
        "deployment disabled, uuid: {}, domain: {}",
        query.uuid, deployment.domain
    );
    Redirect::to(format!("/projects/{}", param).as_str())
}

#[derive(Debug, Deserialize)]
pub struct ProjectRenameParams {
    name: String,
}

#[instrument(skip_all, "project_rename")]
pub async fn handle_rename(
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
    Form(query): Form<ProjectRenameParams>,
) -> Redirect {
    let project = project::find_by_name(param.clone(), current_user.id)
        .await
        .unwrap();
    if project.is_none() {
        error!("project not found, name: {}", param);
        return Redirect::to(format!("/projects/{}", param).as_str());
    }
    let _ = project::rename(current_user.id, param.clone(), query.name.clone())
        .await
        .unwrap();
    info!("project renamed, old: {}, new: {}", param, query.name);
    Redirect::to(format!("/projects/{}/settings", query.name).as_str())
}

#[derive(Debug, Deserialize)]
pub struct ProjectDeleteParams {
    name: String,
    uuid: String,
}

#[instrument(skip_all, "project_delete")]
pub async fn handle_delete(
    Path(param): Path<String>,
    Extension(current_user): Extension<SessionUser>,
    Form(query): Form<ProjectDeleteParams>,
) -> Redirect {
    let project = project::find_by_name(param.clone(), current_user.id)
        .await
        .unwrap();
    if project.is_none() {
        error!("project not found, name: {}", param);
        return Redirect::to(format!("/projects/{}/settings", param).as_str());
    }
    let project = project.unwrap();
    if project.name != query.name || project.uuid != query.uuid {
        error!(
            "project name or uuid not match, name: {}, uuid: {}",
            query.name, query.uuid
        );
        return Redirect::to(format!("/projects/{}/settings", param).as_str());
    }
    project::remove_project(current_user.id, project.uuid)
        .await
        .unwrap();
    info!("project deleted, name: {}", query.name);
    Redirect::to("/projects")
}
