use crate::{DEFAULT_WASM, ENDPOINT_NAME};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::Serialize;

#[derive(Default, Clone, Serialize, Debug)]
pub struct WorkerInfo {
    pub req_id: String,
    pub wasm_module: String,
    pub user_id: String,
    pub project_id: String,
    pub deploy_id: String,
    pub host: String,
    pub endpoint: String,
}

/// worker_info to get worker info
pub async fn worker_info(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let req_id = xid::new().to_string();
    let headers = request.headers().clone();

    // get module path
    let default_wasm_path = DEFAULT_WASM.get().unwrap();
    let wasm_module = headers
        .get("x-land-m")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(default_wasm_path.as_str())
        .to_string();

    // get user-id, project-id, deploy-id, host
    let user_id = headers
        .get("x-land-uid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0")
        .to_string();
    let project_id = headers
        .get("x-land-pid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0")
        .to_string();
    let deploy_id = headers
        .get("x-land-did")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0")
        .to_string();
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let endpoint = ENDPOINT_NAME.get().unwrap().to_string();
    let info = WorkerInfo {
        req_id,
        wasm_module,
        user_id,
        project_id,
        deploy_id,
        host,
        endpoint,
    };

    request.extensions_mut().insert(info);
    Ok(next.run(request).await)
}
