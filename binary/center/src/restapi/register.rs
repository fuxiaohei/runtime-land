use super::{params, AppError};
use crate::email;
use axum::extract::Path;
use axum::{http::StatusCode, Json};
use tracing::{debug_span, info, Instrument};
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

#[tracing::instrument(name = "[forget_password]", skip_all)]
pub async fn forget_password(
    Json(payload): Json<params::ForgetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    let token = land_dao::user::forget_password(&payload.email).await?;
    let link = format!("{}/reset-password/{}", payload.base, token.value);
    info!("success, email:{}, token:{}", payload.email, link,);
    tokio::task::spawn(
        async move {
            email::send_forget_password_email(payload.email, link).await;
        }
        .instrument(debug_span!("[send_email]")),
    );
    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "[reset_password]", skip_all)]
pub async fn reset_password(
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<params::ResetPasswordResponse>), AppError> {
    let (password, email) = land_dao::user::reset_password(token).await?;
    info!("success, email:{}", email,);
    Ok((
        StatusCode::OK,
        Json(params::ResetPasswordResponse { email, password }),
    ))
}
