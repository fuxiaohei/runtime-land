use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::ServerError;
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::admin::{DeployDetailVars, DeployVars};
use land_core_service::vars::{admin, PageVars, PaginationVar};
use land_dao::deployment::{self, DeployStatus, DeploymentStatus};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DeploysQuery {
    domain: Option<String>,
    #[serde(rename = "common-status")]
    common_status: Option<String>,
    #[serde(rename = "deploy-status")]
    deploy_status: Option<String>,
    page: Option<u64>,
    size: Option<u64>,
}

impl DeploysQuery {
    pub fn to_deploy_status_filter(&self) -> Vec<DeployStatus> {
        let mut filters = vec![];
        if let Some(status) = &self.deploy_status {
            if status == "all" {
                return filters;
            }
            if let Ok(s) = status.parse::<DeployStatus>() {
                filters.push(s);
            }
        }
        filters
    }
    pub fn to_common_status_filter(&self) -> Vec<DeploymentStatus> {
        let mut filters = vec![];
        if let Some(status) = &self.common_status {
            if let Ok(s) = status.parse::<DeploymentStatus>() {
                filters.push(s);
            }
        }
        if filters.is_empty() {
            filters.push(DeploymentStatus::Active);
            filters.push(DeploymentStatus::Disabled);
        }
        filters
    }
    pub fn to_query_string(&self) -> String {
        let mut query = String::new();
        if let Some(domain) = &self.domain {
            query.push_str(&format!("domain={}&", domain));
        }
        if let Some(common_status) = &self.common_status {
            query.push_str(&format!("common-status={}&", common_status));
        }
        if let Some(deploy_status) = &self.deploy_status {
            query.push_str(&format!("deploy-status={}&", deploy_status));
        }
        query
    }
}

/// index is a handler for GET /admin/deploys
pub async fn index(
    Extension(user): Extension<SessionUser>,
    engine: template::Engine,
    Query(q): Query<DeploysQuery>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        deploys: Vec<admin::DeployVars>,
        pagination: PaginationVar,
        deploy_status_list: Vec<admin::DeployStatusVars>,
        common_status_list: Vec<admin::DeployCommonStatusVars>,
    }
    let page = q.page.unwrap_or(1);
    let page_size = q.size.unwrap_or(10);
    let (dps, pages) = deployment::list_by_status_paginate(
        q.to_common_status_filter(),
        q.to_deploy_status_filter(),
        page,
        page_size,
        q.domain.clone(),
    )
    .await?;
    let deploys = admin::DeployVars::from_models(dps).await?;
    let pagination = PaginationVar::new(
        page,
        page_size,
        pages.number_of_items,
        pages.number_of_pages,
        format!("/deploys?{}", q.to_query_string()).as_str(),
    );
    let deploy_status_list =
        admin::DeployStatusVars::new_list(&q.deploy_status.unwrap_or_default());
    let common_status_list =
        admin::DeployCommonStatusVars::new_list(&q.common_status.unwrap_or_default());
    Ok(RenderHtmlMinified(
        "deploys.hbs",
        engine,
        Vars {
            page: PageVars::new_admin("Deploys", "admin-deploys"),
            user,
            deploys,
            pagination,
            deploy_status_list,
            common_status_list,
        },
    ))
}

/// details is a handler for GET /admin/deploys/details/:id
pub async fn details(
    Extension(user): Extension<SessionUser>,
    engine: template::Engine,
    Path(deploy_id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        deploy: DeployVars,
        details: Vec<admin::DeployDetailVars>,
    }
    let dp = deployment::get_by_id(deploy_id).await?;
    if dp.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Deployment not found",
        ));
    }
    let deploy = DeployVars::from_model(dp.unwrap()).await?;
    let tasks = deployment::list_tasks_by_deploy_id(deploy_id).await?;
    let details = DeployDetailVars::from_models(tasks).await?;
    Ok(RenderHtmlMinified(
        "deploys-details.hbs",
        engine,
        Vars {
            page: PageVars::new_admin("Deploys", "admin-deploys"),
            user,
            deploy,
            details,
        },
    ))
}
