use super::{auth::CurrentUser, params, AppError};
use axum::{http::StatusCode, Extension, Json};
use land_dao::settings;
use std::collections::HashMap;
use tracing::info;
use validator::Validate;

fn is_admin(user: &CurrentUser) -> Result<(), AppError> {
    if !user.is_admin() {
        return Err(AppError(
            anyhow::anyhow!("permission denied"),
            StatusCode::FORBIDDEN,
        ));
    }
    Ok(())
}

/// list_regions lists all regions
#[tracing::instrument(name = "[list_regions]", skip_all)]
pub async fn list_regions(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::RegionResponse>>), AppError> {
    is_admin(&current_user)?;
    let regions = land_dao::region::list().await?;
    let values: Vec<params::RegionResponse> = regions
        .into_iter()
        .map(|region| params::RegionResponse {
            id: region.id,
            name: region.name,
            key: region.key,
            runtimes: region.runtimes,
            status: region.status,
        })
        .collect();
    info!("success, count:{}", values.len());
    Ok((StatusCode::OK, Json(values)))
}

/// list_settings_domains lists production domains settings
#[tracing::instrument(name = "[list_settings_domains]", skip_all)]
pub async fn list_settings_domains(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<HashMap<String, String>>), AppError> {
    is_admin(&current_user)?;
    let keys = vec![
        settings::Key::ProductionDomain.to_string(),
        settings::Key::ProductionProtocol.to_string(),
    ];
    let settings = land_dao::settings::list_maps(keys).await?;
    info!("success, count:{}", settings.len());
    Ok((StatusCode::OK, Json(settings)))
}

#[tracing::instrument(name = "[update_settings_domain]", skip_all)]
pub async fn update_settings_domain(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::SettingsDomainRequest>,
) -> Result<StatusCode, AppError> {
    is_admin(&current_user)?;
    payload.validate()?;
    crate::settings::update_domains(payload.domain.clone(), payload.protocol.clone()).await?;
    info!(
        "success, domain:{}, protocol:{}",
        payload.domain, payload.protocol
    );
    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "[list_settings_storage]", skip_all)]
pub async fn list_settings_storage(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<params::SettingsStorageResponse>), AppError> {
    is_admin(&current_user)?;
    let (key, local_config, s3_config) = crate::settings::load_storage_settings().await?;
    let response = params::SettingsStorageResponse {
        storage_type: key.clone(),
        local: local_config,
        s3: s3_config,
    };
    info!("success, key:{}", key);
    Ok((StatusCode::OK, Json(response)))
}

#[tracing::instrument(name = "[update_settings_storage]", skip_all)]
pub async fn update_settings_storage(
    Extension(current_user): Extension<CurrentUser>,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
    is_admin(&current_user)?;
    let config = serde_json::from_slice::<land_storage::S3Config>(&body)?;
    crate::settings::reload_s3(&config).await?;
    info!("success, config:{:?}", config);
    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "[stats_handler]", skip_all)]
pub async fn stats_handler(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<params::StatsResponse>), AppError> {
    is_admin(&current_user)?;
    let response = params::StatsResponse {
        deployments: land_dao::deployment::get_stats().await?,
        projects: land_dao::project::get_stats().await?,
        users: land_dao::user::get_stats().await?,
        regions: land_dao::region::get_stats().await?,
    };
    Ok((StatusCode::OK, Json(response)))
}

/// list_tokens_for_region lists all tokens of current user.
#[tracing::instrument(name = "[list_tokens_for_region]", skip_all)]
pub async fn list_tokens_for_region(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::TokenResponse>>), AppError> {
    is_admin(&current_user)?;
    let tokens = land_dao::user_token::list_by_created(
        current_user.id,
        land_dao::user_token::CreatedByCases::Edgehub,
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
            value: t.value,
        })
        .collect();
    info!(
        "list_tokens_for_region success, count:{}, userid: {}",
        values.len(),
        current_user.id
    );
    Ok((StatusCode::OK, Json(values)))
}

#[tracing::instrument(name = "[create_token_for_region]", skip_all)]
pub async fn create_token_for_region(
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<params::CreateRegionTokenRequest>,
) -> Result<(StatusCode, Json<params::TokenResponse>), AppError> {
    is_admin(&current_user)?;

    let token = land_dao::user_token::find_by_name(
        current_user.id,
        payload.name.clone(),
        land_dao::user_token::CreatedByCases::Edgehub,
    )
    .await?;
    if token.is_some() {
        return Err(anyhow::anyhow!("token name is exist").into());
    }

    let token = land_dao::user_token::create(
        current_user.id,
        payload.name.clone(),
        365 * 24 * 60 * 60, // 1 year
        land_dao::user_token::CreatedByCases::Edgehub,
    )
    .await?;

    info!(
        "create_token_for_region success, userid:{}, name:{}",
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
