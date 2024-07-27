use super::ServerError;
use axum::{response::IntoResponse, Extension, Form, Json};
use land_core::traffic;
use land_vars::AuthUser;
use tracing::info;

/// requests is route of traffic requests query page, /traffic/requests
pub async fn requests(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<traffic::Form>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let pid = f.pid.map(|pid| pid.to_string());
    let period = traffic::PeriodParams::new(&f.period, None);
    let lines = traffic::requests_traffic(pid, Some(user.id.to_string()), &period).await?;
    info!(
        "requests, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(lines))
}

/// flows is route of traffic requests query page, /traffic/flows
pub async fn flows(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<traffic::Form>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let pid = f.pid.map(|pid| pid.to_string());
    let period = traffic::PeriodParams::new(&f.period, None);
    let lines = traffic::flow_traffic(pid, Some(user.id.to_string()), &period).await?;
    info!(
        "flows, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(lines))
}

/// flows is route of traffic requests query page, /traffic/flows
pub async fn projects(
    Extension(user): Extension<AuthUser>,
    Json(f): Json<traffic::ProjectsQueryForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let pids = f
        .pids
        .iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<String>>();
    let period = traffic::PeriodParams::new(&f.period, None);
    let lines = traffic::projects_traffic(user.id.to_string(), pids, &period).await?;
    info!(
        "projects, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(lines))
}
