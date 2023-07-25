use super::auth::CurrentUser;
use super::{params, AppError};
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum::{Extension, Json};
use land_dao::user_token;
use tracing::{debug, info};

pub async fn middleware<B>(mut request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = request.headers().get("authorization");
    if auth_header.is_none() {
        info!("no auth header");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_token = auth_header.unwrap().to_str().unwrap();
    let auth_token = auth_token.replace("Bearer ", "");
    let auth_token = auth_token.trim();
    let token = user_token::find(auth_token.to_string())
        .await
        .map_err(|e| {
            info!("find token error: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    if token.is_none() {
        info!("token not found");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = token.unwrap();
    if token.created_by != user_token::CreatedByCases::Edgehub.to_string() {
        info!("token created by not edgehub");
        return Err(StatusCode::UNAUTHORIZED);
    }
    request
        .extensions_mut()
        .insert(CurrentUser { id: token.owner_id });

    let response = next.run(request).await;
    Ok(response)
}

/// sync_handler syncs the region info from edge to center.
pub async fn sync_handler(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::SyncData>,
) -> Result<(), AppError> {
    debug!(
        "sync_handler begin, payload:{:?}, user:{:?}",
        payload, current_user
    );
    Err(AppError(
        anyhow::anyhow!("not implemented"),
        StatusCode::NOT_IMPLEMENTED,
    ))
}
