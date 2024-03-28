use super::auth::SessionUser;
use crate::server::ServerError;
use axum::{extract::Query, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestsQuery {
    pub account: Option<String>,
    pub project: Option<String>,
    pub period: Option<String>, //only for daily or weekly
}

impl RequestsQuery {
    pub fn get_period(&self) -> (i64, i64, i64, String) {
        if let Some(p) = self.period.as_ref() {
            if p.eq("weekly") {
                let end = chrono::Utc::now().timestamp() / 3600 * 3600;
                let start = end - 604800; // 86400 * 7
                return (start, end, 3600, "1h".to_string());
            }
        }
        let end = chrono::Utc::now().timestamp() / 600 * 600;
        let start = end - 86400;
        (start, end, 600, "10m".to_string())
    }
}

pub async fn flows(
    Extension(user): Extension<SessionUser>,
    Query(q): Query<RequestsQuery>,
) -> Result<impl IntoResponse, ServerError> {
    Ok("flows".to_string())
}

/// requests is a handler for GET /traffic/requests
pub async fn requests(
    Extension(user): Extension<SessionUser>,
    Query(q): Query<RequestsQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let (start, end, step, step_word) = q.get_period();
    let acc = q.account.unwrap_or_default();
    if acc != user.id.to_string() {
        return Err(ServerError::forbidden("user id does not match"));
    }
    let query = if let Some(pid) = q.project {
        format!(
            "increase(req_fn_total{{project_id=\"{}\"}}[{}])",
            pid, step_word
        )
    } else {
        format!(
            "sum(increase(req_fn_total{{user_id=\"{}\"}}[{}]))",
            acc, step_word
        )
    };
    // end time is now ts with latest 10 decade
    debug!(
        "query: {}, start:{}, end:{}, step:{}",
        query, start, end, step
    );
    let params = land_kernel::prom::QueryRangeParams {
        query,
        step,
        start,
        end,
    };
    let res = land_kernel::prom::query_range(params).await?;
    Ok(Json(res).into_response())
}
