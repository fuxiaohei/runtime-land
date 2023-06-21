use axum::{http::Request, http::StatusCode, middleware::Next, response::Response};
use land_core::dao;
use tracing::info;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: i32,
}

pub async fn auth<B>(mut request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = request.headers().get("authorization");
    if auth_header.is_none() {
        info!("no auth header");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_token = auth_header.unwrap().to_str().unwrap();
    let auth_token = auth_token.replace("Bearer ", "");
    let auth_token = auth_token.trim();
    let token = dao::token::find(auth_token.to_string())
        .await
        .map_err(|e| {
            info!("find token error: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    if token.is_none() {
        info!("token not found");
        return Err(StatusCode::UNAUTHORIZED);
    }
    request.extensions_mut().insert(CurrentUser {
        id: token.unwrap().owner_id,
    });

    let response = next.run(request).await;
    Ok(response)
}
