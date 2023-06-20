use crate::{params, AppError};
use axum::http::StatusCode;
use axum::Json;
use gravatar::{Gravatar, Rating};
use land_core::dao;
use tracing::info;
use validator::Validate;

pub async fn signup_email(
    Json(payload): Json<params::SignupEmailRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!(
        "signup_email begin, email:{}, nickname:{}",
        payload.email, payload.nickname
    );

    let (user, token) =
        dao::user::signup_by_email(payload.email, payload.password, payload.nickname).await?;
    let gravatar_url = Gravatar::new(&user.email)
        .set_size(Some(400))
        .set_rating(Some(Rating::Pg))
        .image_url();
    info!(
        "signup_email success, email:{}, nickname:{}",
        user.email, user.display_name,
    );

    Ok((
        StatusCode::CREATED,
        Json(params::LoginResponse {
            access_token: token.token,
            access_token_uuid: token.uuid,
            display_name: user.display_name,
            display_email: user.email,
            avatar_url: gravatar_url.to_string(),
        }),
    ))
}

pub async fn login_by_email(
    Json(payload): Json<params::LoginEmailRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!("login_by_email begin, email:{}", payload.email);

    let (user, token) = dao::user::login_by_email(payload.email, payload.password).await?;

    let gravatar_url = Gravatar::new(&user.email)
        .set_size(Some(400))
        .set_rating(Some(Rating::Pg))
        .image_url();
    info!(
        "login_by_email success, email:{}, nickname:{}",
        user.email, user.display_name,
    );

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse {
            access_token: token.token,
            access_token_uuid: token.uuid,
            display_name: user.display_name,
            display_email: user.email,
            avatar_url: gravatar_url.to_string(),
        }),
    ))
}

pub async fn login_by_token(
    Json(payload): Json<params::LoginAccessTokenRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!("login_by_token begin, token:{}", payload.access_token);
    let (user, token) = dao::user::login_by_access_token(payload.access_token).await?;
    info!(
        "login_by_token success, email:{}, nickname:{}",
        user.email, user.display_name,
    );

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse {
            access_token: token.token,
            access_token_uuid: token.uuid,
            display_name: user.display_name,
            display_email: user.email,
            avatar_url: String::new(),
        }),
    ))
}

