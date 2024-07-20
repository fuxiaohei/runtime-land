use axum::{
    extract::{OriginalUri, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use land_dao::tokens;
use tracing::warn;

/// auth is a middleware to check if the worker is authenticated
pub async fn auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let path = if let Some(path) = request.extensions().get::<OriginalUri>() {
        // This will include nested routes
        path.0.path().to_owned()
    } else {
        request.uri().path().to_owned()
    };

    let auth_header = request.headers().get("Authorization");
    if auth_header.is_none() {
        warn!(path = path, "No authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_value = auth_header
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("Bearer ");
    if auth_value.is_empty() {
        warn!(path = path, "Authorization header is empty");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = match tokens::get_by_value(auth_value, Some(tokens::Usage::Worker)).await {
        Ok(t) => t,
        Err(e) => {
            warn!(path = path, "Error getting token: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    if token.is_none() {
        warn!(path = path, "Token not found");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = token.unwrap();
    if tokens::is_expired(&token) {
        warn!(path = path, "Token is expired");
        return Err(StatusCode::UNAUTHORIZED);
    }
    if token.status != tokens::Status::Active.to_string() {
        warn!(path = path, "Token is not active");
        return Err(StatusCode::UNAUTHORIZED);
    }
    // check if the token is used in the last 60 seconds
    if chrono::Utc::now().timestamp() - token.latest_used_at.and_utc().timestamp() > 60 {
        match tokens::set_usage_at(token.id).await {
            Ok(_) => {}
            Err(e) => {
                warn!(path = path, "Error update usage at: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
    Ok(next.run(request).await)
}
