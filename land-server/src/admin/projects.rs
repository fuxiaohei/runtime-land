use crate::{dash::ServerError, templates::Engine};
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use axum_template::RenderHtml;
use land_core::traffic;
use land_dao::{playground, projects};
use land_vars::{AuthUser, BreadCrumbKey, Page, Pagination, Project};
use serde::{Deserialize, Serialize};
use tracing::info;

pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub projects: Vec<Project>,
        pub pagination: Pagination,
    }

    let (project_models, pager) = projects::list(None, None, 1, 20).await?;
    let projects = Project::new_from_models(project_models, true).await?;
    let pagination = Pagination::new(
        1,
        20,
        pager.number_of_pages,
        pager.number_of_items,
        "/admin/projects",
    );

    Ok(RenderHtml(
        "admin/projects.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin Projects", BreadCrumbKey::AdminProjects, Some(user)),
            projects,
            pagination,
        },
    ))
}

/// flows is route of traffic requests query page, /admin/projects/traffic/
pub async fn traffic(
    Json(f): Json<traffic::ProjectsQueryForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let pids = f
        .pids
        .iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<String>>();
    let period = traffic::PeriodParams::new(&f.period, None);
    let lines = traffic::projects_traffic(None, pids, &period).await?;
    info!(
        "admin-projects, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(lines))
}

#[derive(Deserialize, Debug)]
pub struct SourceQuery {
    pub pid: i32,
}

/// source is route of show playground source page, /admin/projects/source
pub async fn source(Query(query): Query<SourceQuery>) -> Result<impl IntoResponse, ServerError> {
    let playground = playground::get_by_project(query.pid).await?;
    if playground.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Playground not found",
        ));
    }
    Ok(playground.unwrap().source.into_response())
}
