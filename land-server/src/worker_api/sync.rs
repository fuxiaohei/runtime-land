use super::{response_ok, JsonError};
use axum::{
    body::{Body, HttpBody},
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
};
use land_core::agent;

/// handle /worker-api/sync
pub async fn handle(req: Request<Body>) -> Result<impl IntoResponse, JsonError> {
    let (parts, body) = req.into_parts();
    if body.size_hint().lower() > 0 {
        // refresh living worker agent
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
        let ipinfo = serde_json::from_slice::<agent::IP>(&body_bytes)?;
        agent::set_living(ipinfo).await;
    }

    // check confs md5
    let confs = agent::get_confs().await;
    let req_md5 = parts.headers.get("X-Md5");
    if let Some(req_md5) = req_md5 {
        if req_md5.to_str().unwrap() == confs.0 && !confs.0.is_empty() {
            return Ok((StatusCode::NOT_MODIFIED, ()).into_response());
        }
    }
    
    // if not match, return new confs
    let mut resp = response_ok(confs.1, None).into_response();
    resp.headers_mut()
        .insert("X-Md5", HeaderValue::from_str(confs.0.as_str())?);
    Ok(resp)
}
