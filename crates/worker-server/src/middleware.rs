use super::{DEFAULT_WASM, ENDPOINT_NAME};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;

#[derive(Default, Clone, Serialize, Debug)]
pub struct WorkerContext {
    pub req_id: String,
    pub wasm_module: String,
    pub user_id: String,
    pub project_id: String,
    pub host: String,
    pub endpoint: String,
}

/// middleware to add worker context info to request
pub async fn middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let req_id = ulid::Ulid::new().to_string();
    let headers = request.headers().clone();

    // get wasm path from x-land-module header
    let default_wasm_path = DEFAULT_WASM.get().unwrap();
    let wasm_module = headers
        .get("x-land-module")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(default_wasm_path.as_str())
        .to_string();

    // get user-uuid, project-uuid, host
    let user_id = headers
        .get("x-land-user-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let project_id = headers
        .get("x-land-project-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let endpoint = ENDPOINT_NAME.get().unwrap().to_string();
    let context = WorkerContext {
        req_id,
        wasm_module,
        user_id,
        project_id,
        host,
        endpoint,
    };
    request.extensions_mut().insert(context);
    Ok(next.run(request).await)
}
