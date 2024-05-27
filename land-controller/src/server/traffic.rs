use axum::{response::IntoResponse, Form, Json};
use land_core_service::{
    httputil::ServerError,
    metrics::traffic::{query_total_flow, query_total_requests, TrafficPeriodParams},
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficQueryForm {
    pub period: Option<String>,
}

impl TrafficQueryForm {
    // convert query to period params
    fn get_period(&self) -> TrafficPeriodParams {
        let st = chrono::Utc::now().timestamp();
        let period = self.period.as_deref().unwrap_or("1d");
        TrafficPeriodParams::new(period, Some(st))
    }
}

/// requests is a handler for GET /traffic/requests
pub async fn requests(Form(q): Form<TrafficQueryForm>) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let period = q.get_period();
    let values = query_total_requests(&period).await?;
    info!(
        "total-requests, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(values))
}

/// flows is a handler for GET /traffic/flows
pub async fn flows(Form(q): Form<TrafficQueryForm>) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let period = q.get_period();
    let values = query_total_flow(&period).await?;
    info!(
        "total-flow, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(values))
}
