use super::SessionUser;
use crate::server::ServerError;
use axum::{response::IntoResponse, Extension, Form, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// RequestsForm is a query for requests metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestsForm {
    pub account: Option<String>,
    pub project: Option<String>,
    pub period: Option<String>, //only for daily or weekly
}

#[derive(Debug)]
struct PeriodParams {
    start: i64,
    end: i64,
    step: i64,
    step_word: String,
    sequence: Vec<i64>, // unix timestamp from start to end with step
}

impl RequestsForm {
    // convert query to period params
    fn get_period(&self) -> PeriodParams {
        let st = chrono::Utc::now().timestamp();
        if let Some(p) = self.period.as_ref() {
            if p.eq("7d") {
                let end = (st + 3599) / 3600 * 3600; // add 3500 for round hour, use next hour
                let start = end - 604800; // 86400 * 7 + 2
                let sequence = (0..85).map(|i| start + i * 3600 * 2).collect();
                return PeriodParams {
                    start,
                    end,
                    step: 3600,
                    step_word: "2h".to_string(),
                    sequence,
                };
            }
        }
        let end = (st + 599) / 600 * 600; // use next 10 min
        let start = end - 86400; // oneday 1440/10+1
        let sequence = (0..145).map(|i| start + i * 600).collect();
        PeriodParams {
            start,
            end,
            step: 600,
            step_word: "10m".to_string(),
            sequence,
        }
    }
}

fn get_request_query(acc: String, project: Option<String>, step: String) -> String {
    if let Some(pid) = project {
        "sum(increase(req_fn_total{p_uuid='".to_string()
            + &pid
            + "',status='all'}["
            + step.as_str()
            + "]))"
    } else {
        "sum(increase(req_fn_total{uuid='".to_string()
            + &acc
            + "',status='all'}["
            + step.as_str()
            + "]))"
    }
}

/// requests is a handler for GET /traffic/requests
pub async fn requests(
    Extension(_user): Extension<SessionUser>,
    Form(q): Form<RequestsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let period = q.get_period();
    let acc = q.account.unwrap_or_default();
    /*
    FIXME: user.uuid is testing data, need to be replaced by real user uuid
    if acc != user.uuid.to_string() {
        return Err(ServerError::status_code(
            StatusCode::FORBIDDEN,
            "User uuid does not match",
        ));
    }*/
    let query = get_request_query(acc, q.project, period.step_word);
    // end time is now ts with latest 10 decade
    debug!(
        "query: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    let params = land_core::metrics::QueryRangeParams {
        query: query.clone(),
        step: period.step,
        start: period.start,
        end: period.end,
    };
    let res = land_core::metrics::query_range(params).await?;
    let values = land_core::metrics::LineSeries::from(&res, period.sequence);
    info!(
        "query: {}, start:{}, end:{}, step:{}, cost:{}",
        query,
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(values))
}

fn get_flow_query(acc: String, project: Option<String>, step: String) -> String {
    if let Some(pid) = project {
        "sum by (p_uuid,flowtype) (increase(req_fn_bytes_total{p_uuid='".to_string()
            + &pid
            + "'}["
            + step.as_str()
            + "]))"
    } else {
        "sum by (uuid,flowtype) (increase(req_fn_bytes_total{uuid='".to_string()
            + &acc
            + "'}["
            + step.as_str()
            + "]))"
    }
}

/// flows is a handler for GET /traffic/flows
pub async fn flows(
    Extension(_user): Extension<SessionUser>,
    Form(q): Form<RequestsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let period = q.get_period();
    let acc = q.account.unwrap_or_default();
    /*
    FIXME: user.uuid is testing data, need to be replaced by real user uuid
    if acc != user.uuid.to_string() {
        return Err(ServerError::status_code(
            StatusCode::FORBIDDEN,
            "User uuid does not match",
        ));
    }*/
    let query = get_flow_query(acc, q.project, period.step_word);
    // end time is now ts with latest 10 decade
    debug!(
        "query: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    let params = land_core::metrics::QueryRangeParams {
        query: query.clone(),
        step: period.step,
        start: period.start,
        end: period.end,
    };
    let res = land_core::metrics::query_range(params).await?;
    let values = land_core::metrics::LineSeries::from(&res, period.sequence);
    info!(
        "query: {}, start:{}, end:{}, step:{}, cost:{}",
        query,
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(values))
}
