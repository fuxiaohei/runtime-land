use super::{auth::CurrentUser, params, AppError};
use crate::conf;
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
    let map_values: HashMap<String, String> = vec![
        (
            settings::Key::ProductionDomain.to_string(),
            payload.domain.clone(),
        ),
        (
            settings::Key::ProductionProtocol.to_string(),
            payload.protocol.clone(),
        ),
    ]
    .into_iter()
    .collect();
    land_dao::settings::update_maps(map_values).await?;
    conf::trigger().await;
    info!(
        "success, domain:{}, protocol:{}",
        payload.domain, payload.protocol
    );
    Ok(StatusCode::OK)
}
