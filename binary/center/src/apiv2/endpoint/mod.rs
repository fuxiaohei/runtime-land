use super::RouteError;
use axum::{
    middleware::{self, Next},
    response,
    routing::get,
    Router,
};
use hyper::{Request, StatusCode};
use land_dao::user_token;
use tracing::error;

mod conf;

pub fn router() -> Router {
    Router::new()
        .route("/v2/endpoint/conf", get(conf::conf_handler))
        .route_layer(middleware::from_fn(auth_middleware))
}

#[derive(Clone, Debug)]
pub struct SessionUser {
    pub id: i32,
    pub role: String,
}

pub async fn auth_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<response::Response, StatusCode> {
    let auth_header = request.headers().get("authorization");
    if auth_header.is_none() {
        error!("no authorization header or bear token");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_token = auth_header.unwrap().to_str().unwrap();
    let auth_token = auth_token.replace("Bearer ", "");
    let auth_token = auth_token.trim();
    let (token, user) = user_token::find_by_value_with_active_user(auth_token.to_string())
        .await
        .map_err(|e| {
            error!("auth, find token error: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    if token.created_by != user_token::CreatedByCases::Edgehub.to_string() {
        error!("token created by not land-edge");
        return Err(StatusCode::UNAUTHORIZED);
    }
    user_token::refresh(token.id).await.map_err(|e| {
        error!("auth, refresh token error: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;
    request.extensions_mut().insert(SessionUser {
        id: token.owner_id,
        role: user.role,
    });
    let response = next.run(request).await;
    Ok(response)
}
