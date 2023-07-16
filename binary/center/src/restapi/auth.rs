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

pub async fn signup_email(
    Json(payload): Json<params::SignupEmailRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!(
        "signup_email begin, email:{}, nickname:{}",
        payload.email, payload.nickname
    );

    let (user, token) =
        land_dao::user::signup_by_email(payload.email, payload.password, payload.nickname).await?;

    info!(
        "signup_email success, email:{}, nickname:{}",
        user.email, user.nick_name,
    );

    Ok((
        StatusCode::CREATED,
        Json(params::LoginResponse {
            access_token: token.value,
            access_token_uuid: token.uuid,
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
        }),
    ))
}

pub async fn login_by_email(
    Json(payload): Json<params::LoginEmailRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!("login_by_email begin, email:{}", payload.email);

    let (user, token) = land_dao::user::login_by_email(payload.email, payload.password).await?;

    info!(
        "login_by_email success, email:{}, nickname:{}",
        user.email, user.nick_name,
    );

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse {
            access_token: token.value,
            access_token_uuid: token.uuid,
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
        }),
    ))
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
            access_token: token.value,
            access_token_uuid: token.uuid,
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
        }),
    ))
}
