use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use axum_csrf::CsrfToken;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::ServerError;
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::{admin, PageVars, PaginationVar};
use land_dao::deployment;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DeploysQuery {
    page: Option<u64>,
    size: Option<u64>,
}

/// index is a handler for GET /admin/deploys
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: template::Engine,
    Query(q): Query<DeploysQuery>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
        deploys: Vec<admin::DeployVars>,
        pagination: PaginationVar,
    }
    let csrf = csrf_layer.authenticity_token()?;
    let page = q.page.unwrap_or(1);
    let page_size = q.size.unwrap_or(10);
    let (dps, pages) = deployment::list_by_status_paginate(
        vec![
            deployment::DeploymentStatus::Active,
            deployment::DeploymentStatus::Disabled,
        ],
        page,
        page_size,
    )
    .await?;
    let deploys = admin::DeployVars::from_models(dps).await?;
    let pagination = PaginationVar::new(
        page,
        page_size,
        pages.number_of_items,
        pages.number_of_pages,
        "/deploys",
    );
    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "deploys.hbs",
            engine,
            Vars {
                page: PageVars::new_admin("Deploys", "admin-deploys"),
                user,
                csrf,
                deploys,
                pagination,
            },
        ),
    )
        .into_response())
}
