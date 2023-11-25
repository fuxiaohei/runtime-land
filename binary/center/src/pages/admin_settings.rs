use super::auth::SessionUser;
use super::vars::{DomainVars, PageVars, RuntimeNodeVars, StorageVars, UserVars};
use super::AppEngine;
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Form};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct AdminEndpointsVars {
    pub page: PageVars,
    pub user: UserVars,
    pub node_count: u64,
    pub nodes: Vec<RuntimeNodeVars>,
}

pub async fn render_runtime_nodes(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new(
        "Runtime Nodes | Admin ".to_string(),
        "/admin/runtime-nodes".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    let nodes = land_dao::runtime_node::list_all().await.unwrap();
    let node_vars = RuntimeNodeVars::from_models(&nodes);
    RenderHtml(
        "admin/runtime_nodes.hbs",
        engine,
        AdminEndpointsVars {
            page: page_vars,
            user: user_vars,
            node_count: nodes.len() as u64,
            nodes: node_vars,
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminStorageVars {
    pub page: PageVars,
    pub user: UserVars,
    pub storage: StorageVars,
}

pub async fn render_storage(
    engine: AppEngine,
    csrf_token: CsrfToken,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page_vars = PageVars::new("Storage | Admin ".to_string(), "/admin/storage".to_string());
    let user_vars = UserVars::new(&current_user);
    let mut storage_vars = StorageVars::load().await.unwrap();
    storage_vars.csrf_token = csrf_token_value.clone();
    (
        csrf_token,
        RenderHtml(
            "admin/storage.hbs",
            engine,
            AdminStorageVars {
                page: page_vars,
                user: user_vars,
                storage: storage_vars,
            },
        ),
    )
        .into_response()
}

pub async fn handle_storage(
    csrf_token: CsrfToken,
    Form(payload): Form<StorageVars>,
) -> Result<Redirect, StatusCode> {
    let span = tracing::info_span!("handle_storage");
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }

    let (storage_type, fs, s3) = payload.to_model();

    // save configs
    fs.save_db().await.unwrap();
    s3.save_db().await.unwrap();
    // save storage type
    land_storage::dao::save_storage_type(storage_type.clone())
        .await
        .unwrap();

    info!("update success, storage_type:{}", storage_type);
    Ok(Redirect::to("/admin/storage"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminDomainVars {
    pub page: PageVars,
    pub user: UserVars,
    pub domains: DomainVars,
}

pub async fn render_domains(
    engine: AppEngine,
    csrf_token: CsrfToken,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page_vars = PageVars::new("Storage | Admin ".to_string(), "/admin/storage".to_string());
    let user_vars = UserVars::new(&current_user);
    let mut domain_vars = DomainVars::load().await;
    domain_vars.csrf_token = csrf_token_value.clone();
    (
        csrf_token,
        RenderHtml(
            "admin/domains.hbs",
            engine,
            AdminDomainVars {
                page: page_vars,
                user: user_vars,
                domains: domain_vars,
            },
        ),
    )
        .into_response()
}

pub async fn handle_domains(
    csrf_token: CsrfToken,
    Form(payload): Form<DomainVars>,
) -> Result<Redirect, StatusCode> {
    let span = tracing::info_span!("handle_domains");
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }

    // update db
    land_dao::settings::save_domain_protocol(payload.domain.clone(), payload.protocol.clone())
        .await
        .unwrap();
    // update global var in mem
    land_core::confdata::set_domain(payload.domain.clone(), payload.protocol.clone()).await;

    info!(
        "update success, domain:{}, protocol:{}",
        payload.domain, payload.protocol
    );
    Ok(Redirect::to("/admin/domains"))
}
