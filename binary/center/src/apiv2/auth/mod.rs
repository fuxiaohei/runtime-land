use self::params::{CreateTokenRequest, LoginResponse, VerifyTokenRequest};
use super::RouteError;
use axum::routing::post;
use axum::{http::StatusCode, Json, Router};
use land_dao::{user, user_token};
use tracing::{info, warn};
use validator::Validate;

mod params;

pub fn router() -> Router {
    Router::new()
        .route("/v2/auth/verify", post(verify_token))
        .route("/v2/auth/create", post(create_token))
}

#[tracing::instrument(name = "[verify_token]", skip_all)]
pub async fn verify_token(
    Json(payload): Json<VerifyTokenRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), RouteError> {
    let (user, token) = land_dao::user::login_by_token(payload.token).await?;
    info!("success, email:{}, nickname:{}", user.email, user.nick_name,);

    let expire_at = token.expired_at.unwrap().timestamp();

    // if expired ,set unauthorized
    if expire_at < chrono::Utc::now().timestamp() {
        warn!("token expired");
        return Err(RouteError(
            anyhow::anyhow!("token expired"),
            StatusCode::UNAUTHORIZED,
        ));
    }

    user_token::refresh(token.id).await.map_err(|e| {
        warn!("refresh token error: {:?}", e);
        RouteError(
            anyhow::anyhow!("refresh token error"),
            StatusCode::UNAUTHORIZED,
        )
    })?;

    Ok((StatusCode::OK, Json(LoginResponse::new(&user, &token))))
}

#[tracing::instrument(name = "[create_token]", skip(payload))]
pub async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), RouteError> {
    payload.validate()?;
    info!("begin, email:{}", payload.email);

    // get user by oauth_id, if exist, create new token
    let user = user::find_by_oauth_id(payload.oauth_id.clone()).await?;
    if user.is_some() {
        let user = user.unwrap();
        let token = user_token::create(
            user.id,
            String::from("create_oauth_token"),
            60 * 60 * 24 * 10, // 10 days
            user_token::CreatedByCases::OauthLogin,
        )
        .await?;
        info!("success, email:{}, nickname:{}", user.email, user.nick_name,);
        return Ok((StatusCode::OK, Json(LoginResponse::new(&user, &token))));
    }

    let (user, token) = user::signup_by_oauth(
        payload.name,
        payload.display_name,
        payload.email,
        payload.image_url,
        payload.oauth_id,
        payload.oauth_provider,
        payload.oauth_social,
    )
    .await?;
    info!(
        "success by signup_by_oauth, email:{}, nickname:{}",
        user.email, user.nick_name,
    );

    Ok((StatusCode::OK, Json(LoginResponse::new(&user, &token))))
}
