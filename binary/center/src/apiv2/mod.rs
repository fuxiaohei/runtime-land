use axum::{
    body::Body,
    http::{Request, Response},
    middleware::{self, Next},
    response,
    response::IntoResponse,
    routing::any,
    Router,
};
use hyper::StatusCode;
use land_dao::user_token;
use tracing::error;

mod endpoint;

pub fn api_router() -> Router {
    Router::new()
        .merge(endpoint::router())
        .route("/v2/*path", any(v2_handler))
        .route_layer(middleware::from_fn(auth_middleware))
}

async fn v2_handler(_req: Request<Body>) -> Response<Body> {
    let mut builder = Response::builder().status(200);
    builder = builder.header("x-land-version", land_core::version::get());
    builder
        .body(Body::from("Hello, Runtime.land! API v2"))
        .unwrap()
}

// Make our own error that wraps `anyhow::Error`.
pub struct RouteError(anyhow::Error, StatusCode);

#[derive(serde::Serialize)]
struct RouteErrorJson {
    pub message: String,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        error!("app error: {:?}", self.0);
        (
            self.1,
            axum::Json::from(RouteErrorJson {
                message: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, RouteErrorJson>`. That way you don't need to do that manually.
impl<E> From<E> for RouteError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
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
