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

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQueryParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminProjectVars {
    pub page: PageVars,
    pub user: UserVars,
    pub project_count: u64,
    pub projects: Vec<ProjectVars>,
    pub pagination: PaginationVars,
    pub search: String,
    pub csrf_token: String,
}

pub async fn render(
    engine: AppEngine,
    csrf_token: CsrfToken,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<SearchQueryParams>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (projects, pages, alls) =
        project::list_all_available_with_page(query.search.clone(), page, page_size)
            .await
            .unwrap();

    let project_ids: HashSet<i32> = projects.iter().map(|p| p.id).collect();
    let deploy_counts = deployment::list_counter_by_projects(project_ids.into_iter().collect())
        .await
        .unwrap();

    let owner_ids: HashSet<i32> = projects.iter().map(|p| p.owner_id).collect();
    let owners = user::list_by_ids(owner_ids.into_iter().collect())
        .await
        .unwrap();

    let page_vars = PageVars::new(
        "Admin - Projects".to_string(),
        "/admin/projects".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    let project_vars = ProjectVars::from_models(&projects, deploy_counts, owners)
        .await
        .unwrap();
    let pagination_vars = PaginationVars::new(page, pages, "/admin/projects");

    (
        csrf_token,
        RenderHtml(
            "admin/projects.hbs",
            engine,
            AdminProjectVars {
                page: page_vars,
                user: user_vars,
                project_count: alls,
                projects: project_vars,
                pagination: pagination_vars,
                search: query.search.unwrap_or_default(),
                csrf_token: csrf_token_value,
            },
        ),
    )
        .into_response()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProjectVars {
    pub name: String,
    pub language: String,
    pub uuid: String,
    pub deployments: usize,
    pub is_prod: bool,
    pub production_url: String,
    pub production_label: String,
    pub updated_timeago: String,
    pub status_label: String,
    pub status: String,
    pub owner_name: String,
    pub owner_email: String,
    pub owner_id: i32,
    pub is_active: bool,
}

impl ProjectVars {
    pub async fn from_models(
        projects: &Vec<land_dao::Project>,
        counters: HashMap<i32, usize>,
        users: HashMap<i32, land_dao::User>,
    ) -> Result<Vec<ProjectVars>> {
        let (prod_domain, prod_protocol) = confdata::get_domain().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for project in projects {
            let user = users.get(&project.owner_id);
            if user.is_none() {
                continue;
            }
            let user = user.unwrap();

            let counter = counters.get(&project.id).unwrap_or(&0);
            let duration = chrono::Utc::now()
                .signed_duration_since(project.updated_at)
                .add(Duration::seconds(2)); // if duation is zero after updated right now, tago.convert fails

            let mut project_vars = ProjectVars {
                name: project.name.clone(),
                language: project.language.clone(),
                uuid: project.uuid.clone(),
                deployments: *counter,
                production_url: "".to_string(),
                production_label: "".to_string(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status_label: "running".to_string(),
                status: project.status.clone(),
                owner_name: user.nick_name.clone(),
                owner_email: user.email.clone(),
                owner_id: user.id,
                is_prod: project.prod_deploy_id > 0,
                is_active: project.status != Status::InActive.to_string(),
            };
            if project.prod_deploy_id > 0 {
                project_vars.production_url =
                    format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                project_vars.production_label = format!("{}.{}", project.name, prod_domain)
            } else {
                project_vars.status_label = "develop".to_string();
            }
            if *counter == 0 {
                project_vars.status_label = "empty".to_string();
            }
            vars.push(project_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleProjectParams {
    pub csrf_token: String,
    pub uuid: String,
    pub owner_id: i32,
    pub action: String,
    pub name: String,
}

pub async fn handle(
    csrf_token: CsrfToken,
    Form(payload): Form<HandleProjectParams>,
) -> Result<Redirect, StatusCode> {
    let action = payload.action.as_str();
    let name = payload.name.as_str();
    let span = tracing::info_span!("handle_project", action, name);
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }
    let project = match project::find_by_uuid(payload.uuid, payload.owner_id).await {
        Ok(p) => {
            if p.is_none() {
                warn!("project not found");
                return Err(StatusCode::NOT_FOUND);
            }
            p.unwrap()
        }
        Err(err) => {
            warn!("project found error, err:{}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    match payload.action.as_str() {
        "enable" => {
            project::set_active(project.id).await.unwrap();
        }
        "disable" => {
            project::set_inactive(project.id).await.unwrap();
        }
        _ => {}
    }
    info!("project action success");
    Ok(Redirect::to("/admin/projects"))
}
