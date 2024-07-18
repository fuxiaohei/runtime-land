use super::JsonError;
use axum::{body::Body, extract::Request, http::StatusCode, response::IntoResponse};
use land_core::agent;

/// handle /worker-api/sync
pub async fn handle(req: Request<Body>) -> Result<impl IntoResponse, JsonError> {
    let (_parts, body) = req.into_parts();
    // refresh living worker agent
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
    let ipinfo = serde_json::from_slice::<agent::IP>(&body_bytes)?;
    agent::set_living(ipinfo).await;

    return Ok((StatusCode::NO_CONTENT, ()).into_response());
}
