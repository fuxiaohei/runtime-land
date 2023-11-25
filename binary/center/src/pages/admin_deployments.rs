use super::auth::SessionUser;
use super::vars::{PageVars, PaginationVars, UserVars};
use super::AppEngine;
use anyhow::Result;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Form};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use chrono::Duration;
use hyper::StatusCode;
use land_core::confdata;
use land_dao::deployment::Status;
use land_dao::{deployment, project, user};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Add;
use tracing::{info, warn};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DeployVars {
    pub domain: String,
    pub uuid: String,
    pub language: String,
    pub project_name: String,
    pub is_prod: bool,
    pub visit_url: String,
    pub visit_label: String,
    pub updated_timeago: String,
    pub status: String,
    pub owner_name: String,
    pub owner_email: String,
    pub owner_id: i32,
    pub is_active: bool,
}

impl DeployVars {
    pub async fn from_models(
        deploys: &Vec<land_dao::Deployment>,
        projects: HashMap<i32, land_dao::Project>,
        users: HashMap<i32, land_dao::User>,
    ) -> Result<Vec<DeployVars>> {
        let (prod_domain, prod_protocol) = confdata::get_domain().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for deploy in deploys {
            let project = projects.get(&deploy.project_id);
            if project.is_none() {
                continue;
            }
            let project = project.unwrap();
            let user = users.get(&project.owner_id);
            if user.is_none() {
                continue;
            }
            let user = user.unwrap();

            let duration = chrono::Utc::now()
                .signed_duration_since(deploy.updated_at)
                .add(Duration::seconds(2)); // if duation is zero after updated right now, tago.convert fails
            let mut project_vars = DeployVars {
                domain: deploy.domain.clone(),
                uuid: deploy.uuid.clone(),
                language: project.language.clone(),
                project_name: project.name.clone(),
                visit_url: String::new(),
                visit_label: String::new(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status: deploy.status.clone(),
                owner_name: user.nick_name.clone(),
                owner_email: user.email.clone(),
                owner_id: user.id,
                is_prod: project.prod_deploy_id == deploy.id,
                is_active: deploy.status == Status::Active.to_string(),
            };
            if project_vars.is_active {
                project_vars.visit_url =
                    format!("{}://{}.{}", prod_protocol, deploy.domain, prod_domain);
                project_vars.visit_label = format!("{}.{}", deploy.domain, prod_domain);
                if project_vars.is_prod {
                    project_vars.visit_url =
                        format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                    project_vars.visit_label = format!("{}.{}", project.name, prod_domain);
                }
            }
            vars.push(project_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminDeploymentsVars {
    pub page: PageVars,
    pub user: UserVars,
    pub pagination: PaginationVars,
    pub search: String,
    pub deploys_count: u64,
    pub deploys: Vec<DeployVars>,
    pub csrf_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQueryParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub search: Option<String>,
}

pub async fn render(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    csrf_token: CsrfToken,
    Query(query): Query<SearchQueryParams>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (deployments, pages, alls) =
        deployment::list_all_available_with_page(query.search.clone(), page, page_size)
            .await
            .unwrap();

    let owner_ids: HashSet<i32> = deployments.iter().map(|p| p.owner_id).collect();
    let owners = user::list_by_ids(owner_ids.into_iter().collect())
        .await
        .unwrap();

    let project_ids: HashSet<i32> = deployments.iter().map(|p| p.project_id).collect();
    let projects = project::list_by_ids(project_ids.into_iter().collect())
        .await
        .unwrap();

    let deploy_vars = DeployVars::from_models(&deployments, projects, owners)
        .await
        .unwrap();

    let page_vars = PageVars::new(
        "Admin - Deployments".to_string(),
        "/admin/deployments".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    (
        csrf_token,
        RenderHtml(
            "admin/deployments.hbs",
            engine,
            AdminDeploymentsVars {
                page: page_vars,
                user: user_vars,
                pagination: PaginationVars::new(page, pages, "/admin/deployments"),
                search: query.search.unwrap_or_default(),
                deploys_count: alls,
                deploys: deploy_vars,
                csrf_token: csrf_token_value,
            },
        ),
    )
        .into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleDeployParams {
    pub csrf_token: String,
    pub uuid: String,
    pub owner_id: i32,
    pub action: String,
    pub name: String,
}

pub async fn handle(
    csrf_token: CsrfToken,
    Form(payload): Form<HandleDeployParams>,
) -> Result<Redirect, StatusCode> {
    let action = payload.action.as_str();
    let name = payload.name.as_str();
    let span = tracing::info_span!("handle_deploy", action, name);
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }
    let deploy = match deployment::find_by_uuid(payload.owner_id, payload.uuid).await {
        Ok(p) => {
            if p.is_none() {
                warn!("deployment not found");
                return Err(StatusCode::NOT_FOUND);
            }
            p.unwrap()
        }
        Err(err) => {
            warn!("deployment found error,err:{}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    match payload.action.as_str() {
        "enable" => {
            deployment::enable(deploy.owner_id, deploy.uuid)
                .await
                .unwrap();
        }
        "disable" => {
            deployment::disable(deploy.owner_id, deploy.uuid)
                .await
                .unwrap();
        }
        _ => {}
    }
    info!("deployment action success");
    Ok(Redirect::to("/admin/deployments"))
}
