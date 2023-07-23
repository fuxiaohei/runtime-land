use super::{params, AppError};
use axum::{http::Request, http::StatusCode, middleware::Next, response::Response, Json};
use land_dao::user_token;
use tracing::info;
use validator::Validate;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: i32,
}

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
    request.extensions_mut().insert(CurrentUser {
        id: token.unwrap().owner_id,
    });

    let response = next.run(request).await;
    Ok(response)
}

pub async fn verify_token(
    Json(payload): Json<params::LoginTokenRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!("login_by_token begin, token:{}", payload.token);
    let (user, token) = land_dao::user::login_by_token(payload.token).await?;
    info!(
        "login_by_token success, email:{}, nickname:{}",
        user.email, user.nick_name,
    );

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse {
            token_value: token.value,
            token_uuid: token.uuid,
            token_expired_at: token.expired_at.unwrap().timestamp(),
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
            oauth_id: user.oauth_id,
        }),
    ))
}

#[tracing::instrument(name = "[create_token]", skip(payload))]
pub async fn create_token(
    Json(payload): Json<params::CreateTokenRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!("begin, email:{}", payload.email);

    // get user by oauth_id, if exist, create new token
    let user = land_dao::user::find_by_oauth_id(payload.oauth_id.clone()).await?;
    if user.is_some() {
        let user = user.unwrap();
        let token = land_dao::user_token::create(
            user.id,
            String::from("createToken"),
            60 * 60 * 24 * 10, // 10 days
            land_dao::user_token::CreatedByCases::EmailLogin,
        )
        .await?;
        info!("success, email:{}, nickname:{}", user.email, user.nick_name,);
        return Ok((
            StatusCode::OK,
            Json(params::LoginResponse {
                token_value: token.value,
                token_uuid: token.uuid,
                token_expired_at: token.expired_at.unwrap().timestamp(),
                nick_name: user.nick_name,
                email: user.email,
                avatar_url: user.avatar,
                oauth_id: user.oauth_id,
            }),
        ));
    }

    let (user, token) = land_dao::user::signup_by_oauth(
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

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse {
            token_value: token.value,
            token_uuid: token.uuid,
            token_expired_at: token.expired_at.unwrap().timestamp(),
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
            oauth_id: user.oauth_id,
        }),
    ))
}
