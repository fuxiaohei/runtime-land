use crate::AppError;
use anyhow::anyhow;
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// LoginResponse is the response for /cli/login
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_token: String,
    pub user_name: String,
    pub user_uuid: String,
    pub user_email: String,
}

pub async fn login(Path(token): Path<String>) -> Result<Json<LoginResponse>, AppError> {
    let user_token = land_dblayer::user::find_token_by_value(&token).await?;
    if user_token.is_none() {
        warn!("token is not exist, value: {}", token);
        return Err(anyhow!("token is not exist").into());
    }
    let user_token = user_token.unwrap();
    // get current user info
    let user = land_dblayer::user::find_by_id(user_token.owner_id).await?;
    if user.is_none() {
        warn!("user is not exist, id: {}", user_token.owner_id);
        return Err(anyhow!("user is not exist").into());
    }
    let user = user.unwrap();
    // create a new token for cli-accesss for this user
    let now_ts = chrono::Utc::now().timestamp();
    let new_token = land_dblayer::user::create_token(
        user_token.owner_id,
        &format!(
            "cli-access-{}-{}-{}",
            user_token.owner_id, user_token.id, now_ts
        ),
        3600 * 24 * 365,
        land_dblayer::user::TokenCreatedByCases::CliAccess,
    )
    .await?;
    let resp = LoginResponse {
        user_token: new_token.value,
        user_name: user.name,
        user_uuid: user.uuid,
        user_email: user.email,
    };
    debug!("login success, resp: {:?}", resp);
    Ok(Json(resp))
}

/// DeployRequest is the request for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRequest {
    pub metadata: land_common::MetaData,
    pub bundle: Vec<u8>,
    pub user_token: String,
    pub user_uuid: String,
}

pub async fn deploy(Json(payload): Json<DeployRequest>) -> Result<Json<LoginResponse>, AppError> {
    // validate user_token
    let token = land_dblayer::user::find_token_by_value(&payload.user_token).await?;
    if token.is_none() {
        warn!("token is not exist, value: {}", payload.user_token);
        return Err(anyhow!("token is not exist").into());
    }
    let token = token.unwrap();
    if token.created_by != land_dblayer::user::TokenCreatedByCases::CliAccess.to_string() {
        warn!("token is not cli-access, value: {}", payload.user_token);
        return Err(anyhow!("token is not cli-access").into());
    }

    println!("deploying...,payload: {:?}", payload);

    return Err(anyhow!("not implemented").into());
}
