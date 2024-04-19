use super::{DEFAULT_WASM, ENDPOINT_NAME};
use crate::METRICS_ENABLED;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use metrics::{counter, Counter};
use serde::Serialize;

#[derive(Default, Clone, Serialize, Debug)]
pub struct WorkerContext {
    pub req_id: String,
    pub wasm_module: String,
    pub user_uuid: String,
    pub project_uuid: String,
    pub host: String,
    pub endpoint: String,
}

#[derive(Clone)]
pub struct WorkerMetrics {
    pub req_fn_total: Counter,
    pub req_fn_notfound_total: Counter,
    pub req_fn_error_total: Counter,
    pub req_fn_in_bytes_total: Counter,
    pub req_fn_out_bytes_total: Counter,
}

/// middleware to add worker context info to request
pub async fn middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let req_id = xid::new().to_string();
    let headers = request.headers().clone();

    // get wasm path from x-land-module header
    let default_wasm_path = DEFAULT_WASM.get().unwrap();
    let wasm_module = headers
        .get("x-land-m")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(default_wasm_path.as_str())
        .to_string();

    // get user-uuid, project-uuid, host
    let user_uuid = headers
        .get("x-land-uuid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let project_uuid = headers
        .get("x-land-puuid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let endpoint = ENDPOINT_NAME.get().unwrap().to_string();
    let counters = if *METRICS_ENABLED.get().unwrap() {
        let labels = vec![
            ("uuid", user_uuid.clone()),
            ("p_uuid", project_uuid.clone()),
            ("endpoint", endpoint.clone()),
        ];
        let mut all_req_labels = labels.clone();
        all_req_labels.push(("status", "all".to_string()));
        let mut notfound_labels = labels.clone();
        notfound_labels.push(("status", "fn-notfound".to_string()));
        let mut error_req_labels = labels.clone();
        error_req_labels.push(("status", "error".to_string()));
        let mut flow_in_labels = labels.clone();
        flow_in_labels.push(("flowtype", "in".to_string()));
        let mut flow_out_labels = labels.clone();
        flow_out_labels.push(("flowtype", "out".to_string()));

        (
            counter!("req_fn_total", &all_req_labels),
            counter!("req_fn_total", &notfound_labels),
            counter!("req_fn_total", &error_req_labels),
            counter!("req_fn_bytes_total", &flow_in_labels),
            counter!("req_fn_bytes_total", &flow_out_labels),
        )
    } else {
        (
            Counter::noop(),
            Counter::noop(),
            Counter::noop(),
            Counter::noop(),
            Counter::noop(),
        )
    };
    let context = WorkerContext {
        req_id,
        wasm_module,
        user_uuid,
        project_uuid,
        host,
        endpoint,
    };
    let metrics = WorkerMetrics {
        req_fn_total: counters.0,
        req_fn_notfound_total: counters.1,
        req_fn_error_total: counters.2,
        req_fn_in_bytes_total: counters.3,
        req_fn_out_bytes_total: counters.4,
    };
    request.extensions_mut().insert(context);
    request.extensions_mut().insert(metrics);
    Ok(next.run(request).await)
}
