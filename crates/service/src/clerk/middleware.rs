use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use land_dao::user::{TokenUsage, UserStatus};
use reqwest::StatusCode;
use serde::Serialize;
use tracing::warn;

#[derive(Clone, Serialize, Debug)]
pub struct AuthUser {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub email: String,
}

/// middleware is a middleware for session auth
pub async fn middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    let path = uri.path();

    // skip static assets auth
    if path.starts_with("/v1/token") {
        // debug!("auth skip path: {}", path);
        return Ok(next.run(request).await);
    }

    let bearer_auth = request.headers().get("Authorization");
    if bearer_auth.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let bearer_auth = bearer_auth
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("Bearer ");

    // get session by token
    let token =
        match land_dao::user::get_token_by_value(bearer_auth, Some(TokenUsage::Session)).await {
            Ok(token) => token,
            Err(e) => {
                warn!("get_token_by_value error: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
    if token.is_none() {
        warn!("bearer_auth is not exist");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = token.unwrap();
    if land_dao::user::is_token_expired(&token).await {
        warn!("token is expired");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // get user info and fill to request
    let user = match land_dao::user::get_info_by_id(token.user_id, Some(UserStatus::Active)).await {
        Ok(user) => user,
        Err(e) => {
            warn!("get_info_by_id error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    if user.is_none() {
        warn!("user is not exist or invalid");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user = user.unwrap();
    let auth_user = AuthUser {
        id: user.id,
        uuid: user.uuid,
        name: user.name,
        email: user.email,
    };
    request.extensions_mut().insert(auth_user);
    return Ok(next.run(request).await);
}
