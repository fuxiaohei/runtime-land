use super::{params, AppError};
use axum::extract::Path;
use axum::Extension;
use axum::{http::Request, http::StatusCode, middleware::Next, response::Response, Json};
use land_dao::user_token;
use tracing::{info, warn};
use validator::Validate;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: i32,
    pub role: String,
}

impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        self.role == land_dao::user::Role::Admin.to_string()
    }
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
    let (token, user) = user_token::find_by_value_with_active_user(auth_token.to_string())
        .await
        .map_err(|e| {
            warn!("auth, find token error: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    user_token::refresh(token.id).await.map_err(|e| {
        warn!("auth, refresh token error: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;
    request.extensions_mut().insert(CurrentUser {
        id: token.owner_id,
        role: user.role,
    });
    let response = next.run(request).await;
    Ok(response)
}

#[tracing::instrument(name = "[verify_token]", skip_all)]
pub async fn verify_token(
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    let (user, token) = land_dao::user::login_by_token(token).await?;
    info!("success, email:{}, nickname:{}", user.email, user.nick_name,);

    let expire_at = token.expired_at.unwrap().timestamp();

    // if expired ,set unauthorized
    if expire_at < chrono::Utc::now().timestamp() {
        warn!("token expired");
        return Err(AppError(
            anyhow::anyhow!("token expired"),
            StatusCode::UNAUTHORIZED,
        ));
    }

    user_token::refresh(token.id).await.map_err(|e| {
        warn!("refresh token error: {:?}", e);
        AppError(
            anyhow::anyhow!("refresh token error"),
            StatusCode::UNAUTHORIZED,
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(params::LoginResponse {
            token_value: token.value,
            token_uuid: token.uuid,
            token_expired_at: token.expired_at.unwrap().timestamp(),
            token_active_at: token.updated_at.timestamp(),
            token_active_interval: 60,
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
            oauth_id: user.oauth_id,
            role: user.role,
        }),
    ))
}

#[tracing::instrument(name = "[create_oauth_token]", skip(payload))]
pub async fn create_oauth_token(
    Json(payload): Json<params::CreateOauthTokenRequest>,
) -> Result<(StatusCode, Json<params::LoginResponse>), AppError> {
    payload.validate()?;
    info!("begin, email:{}", payload.email);

    // get user by oauth_id, if exist, create new token
    let user = land_dao::user::find_by_oauth_id(payload.oauth_id.clone()).await?;
    if user.is_some() {
        let user = user.unwrap();
        let token = land_dao::user_token::create(
            user.id,
            String::from("create_oauth_token"),
            60 * 60 * 24 * 10, // 10 days
            land_dao::user_token::CreatedByCases::OauthLogin,
        )
        .await?;
        info!("success, email:{}, nickname:{}", user.email, user.nick_name,);
        return Ok((
            StatusCode::OK,
            Json(params::LoginResponse {
                token_value: token.value,
                token_uuid: token.uuid,
                token_expired_at: token.expired_at.unwrap().timestamp(),
                token_active_at: token.updated_at.timestamp(),
                token_active_interval: 60,
                nick_name: user.nick_name,
                email: user.email,
                avatar_url: user.avatar,
                oauth_id: user.oauth_id,
                role: user.role,
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
            token_active_at: token.updated_at.timestamp(),
            token_active_interval: 60,
            nick_name: user.nick_name,
            email: user.email,
            avatar_url: user.avatar,
            oauth_id: user.oauth_id,
            role: user.role,
        }),
    ))
}

/// create_for_deployment creates a new token for current user for deployment
pub async fn create_for_deployment(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::CreateTokenRequest>,
) -> Result<(StatusCode, Json<params::TokenResponse>), AppError> {
    payload.validate()?;

    let token = land_dao::user_token::find_by_name(
        current_user.id,
        payload.name.clone(),
        land_dao::user_token::CreatedByCases::Deployment,
    )
    .await?;
    if token.is_some() {
        return Err(anyhow::anyhow!("token name is exist").into());
    }

    let token = land_dao::user_token::create(
        current_user.id,
        payload.name.clone(),
        365 * 24 * 60 * 60, // 1 year
        land_dao::user_token::CreatedByCases::Deployment,
    )
    .await?;

    info!(
        "create_for_deployment success, userid:{}, name:{}",
        current_user.id, payload.name
    );
    Ok((
        StatusCode::OK,
        Json(params::TokenResponse {
            name: token.name,
            created_at: token.created_at.timestamp(),
            updated_at: token.updated_at.timestamp(),
            expired_at: token.expired_at.unwrap().timestamp(),
            origin: token.created_by.to_string(),
            uuid: token.uuid,
            value: token.value,
        }),
    ))
}

/// list_for_deployment lists all tokens of current user.
pub async fn list_for_deployment(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::TokenResponse>>), AppError> {
    let tokens = land_dao::user_token::list_by_created(
        current_user.id,
        land_dao::user_token::CreatedByCases::Deployment,
    )
    .await?;
    let values: Vec<params::TokenResponse> = tokens
        .into_iter()
        .map(|t| params::TokenResponse {
            name: t.name,
            created_at: t.created_at.timestamp(),
            updated_at: t.updated_at.timestamp(),
            expired_at: t.expired_at.unwrap().timestamp(),
            origin: t.created_by.to_string(),
            uuid: t.uuid,
            value: String::new(),
        })
        .collect();
    info!(
        "list_for_deployment success, count:{}, userid: {}",
        values.len(),
        current_user.id
    );
    Ok((StatusCode::OK, Json(values)))
}

pub async fn remove_token(
    Extension(current_user): Extension<CurrentUser>,
    Path(uuid): Path<String>,
) -> Result<(), AppError> {
    land_dao::user_token::remove(current_user.id, uuid.clone()).await?;
    info!(
        "remove_token success, userid:{}, uuid:{}",
        current_user.id, uuid
    );
    Ok(())
}
