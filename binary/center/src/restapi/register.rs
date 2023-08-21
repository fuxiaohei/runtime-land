use super::{params, AppError};
use axum::{http::StatusCode, Json};
use tracing::info;
use validator::Validate;

#[tracing::instrument(name = "[register]", skip(payload))]
pub async fn register(
    Json(payload): Json<params::SignupRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    let (user, token) =
        land_dao::user::signup_by_email(payload.email, payload.password, payload.nickname).await?;
    info!(
        "success by signup_by_email, email:{}, nickname:{}",
        user.email, user.nick_name,
    );

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse::new(&user, &token)),
    ))
}

#[tracing::instrument(name = "[login]", skip(payload))]
pub async fn login_by_email(
    Json(payload): Json<params::LoginEmailRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    let (user, token) = land_dao::user::login_by_email(payload.email, payload.password).await?;
    info!(
        "success by login_by_email, email:{}, nickname:{}",
        user.email, user.nick_name,
    );
    Ok((
        StatusCode::OK,
        Json(params::LoginResponse::new(&user, &token)),
    ))
}
