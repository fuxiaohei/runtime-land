use axum::{
    middleware::{self, Next},
    response,
    routing::{get, post, put},
    Router,
};
use hyper::{Request, StatusCode};
use land_dao::user_token;
use tracing::error;

mod deployment;
mod params;
mod project;
mod templates;

pub fn router() -> Router {
    Router::new()
        .route("/v2/projects", get(project::list_handler))
        .route("/v2/project", post(project::create_handler))
        .route("/v2/project/:name/overview", get(project::overview_handler))
        .route("/v2/templates", get(templates::list_handler))
        .route("/v2/deployment", post(deployment::create_handler))
        .route("/v2/deployment", put(deployment::update_handler))
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
    if token.created_by != user_token::CreatedByCases::OauthLogin.to_string() {
        error!("token created by not oauth-login");
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
