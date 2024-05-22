use crate::deployer::TrafficPeriodParams;
use axum::{response::IntoResponse, Extension, Form, Json};
use http::StatusCode;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::ServerError;
use serde::{Deserialize, Serialize};
use tracing::info;

/// RequestsForm is a query for requests metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestsForm {
    pub account: Option<String>,
    pub project: Option<String>,
    pub period: Option<String>, //only for daily or weekly
}

impl RequestsForm {
    // convert query to period params
    fn get_period(&self) -> TrafficPeriodParams {
        let st = chrono::Utc::now().timestamp();
        let period = self.period.as_deref().unwrap_or("1d");
        TrafficPeriodParams::new(period, Some(st))
    }
}

/// requests is a handler for GET /traffic/requests
pub async fn requests(
    Extension(user): Extension<SessionUser>,
    Form(q): Form<RequestsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let period = q.get_period();
    let acc = q.account.unwrap_or_default();
    if acc != user.uuid {
        return Err(ServerError::status_code(
            StatusCode::FORBIDDEN,
            "User uuid does not match",
        ));
    }
    let values = crate::deployer::query_requests_traffic(acc, q.project, &period).await?;
    info!(
        "requests, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(values))
}

/// flows is a handler for GET /traffic/flows
pub async fn flows(
    Extension(user): Extension<SessionUser>,
    Form(q): Form<RequestsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let period = q.get_period();
    let acc = q.account.unwrap_or_default();
    if acc != user.uuid {
        return Err(ServerError::status_code(
            StatusCode::FORBIDDEN,
            "User uuid does not match",
        ));
    }
    let values = crate::deployer::query_flows_traffic(acc, q.project, &period).await?;
    info!(
        "flows, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(values))
}
