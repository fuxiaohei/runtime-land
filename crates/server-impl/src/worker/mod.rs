use crate::ServerError;
use anyhow::Result;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use land_dao::user_token;

mod conf;

/// router returns the router for land-worker
pub fn router() -> Result<Router> {
    let app = Router::new()
        .route("/deploys", get(conf::deploys).post(conf::deploys_post))
        .route_layer(middleware::from_fn(middleware));
    Ok(app)
}

async fn middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let bearer_token = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ").map(|b| b.to_string()));
    if bearer_token.is_none() {
        return Ok(ServerError::unauthorized("Unauthorized").into_response());
    }
    let bearer_token = bearer_token.unwrap();
    let token = match user_token::get_by_value(&bearer_token, Some(user_token::Usage::Worker)).await
    {
        Ok(token) => token,
        Err(e) => {
            return Ok(ServerError::internal_error(&e.to_string()).into_response());
        }
    };
    if token.is_none() {
        return Ok(ServerError::unauthorized("Unauthorized").into_response());
    }
    Ok(next.run(request).await)
}
