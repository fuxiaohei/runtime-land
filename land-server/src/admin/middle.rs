use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use land_vars::AuthUser;
use tracing::warn;

/// check_admin checks if user is admin
pub async fn check_admin(request: Request, next: Next) -> Result<Response, StatusCode> {
    let user = request.extensions().get::<AuthUser>();
    if user.is_none() {
        return Ok(crate::dash::redirect("/").into_response());
    }
    let user = user.unwrap();
    if !user.is_admin {
        warn!("User {} is not admin", user.name);
        return Ok(crate::dash::redirect("/").into_response());
    }
    return Ok(next.run(request).await);
}
