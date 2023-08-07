use std::collections::HashMap;

use super::{auth::CurrentUser, params, AppError};
use axum::{http::StatusCode, Extension, Json};
use land_dao::settings;
use tracing::info;

/// list_regions lists all regions
#[tracing::instrument(name = "[list_regions]", skip_all)]
pub async fn list_regions(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::RegionResponse>>), AppError> {
    if !current_user.is_admin() {
        return Err(AppError(
            anyhow::anyhow!("permission denied"),
            StatusCode::FORBIDDEN,
        ));
    }
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

/// list_production_domains lists production domains settings
#[tracing::instrument(name = "[list_production_domains]", skip_all)]
pub async fn list_production_domains(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<HashMap<String, String>>), AppError> {
    if !current_user.is_admin() {
        return Err(AppError(
            anyhow::anyhow!("permission denied"),
            StatusCode::FORBIDDEN,
        ));
    }
    let keys = vec![
        settings::Key::ProductionDomain.to_string(),
        settings::Key::ProductionProtocol.to_string(),
    ];
    let settings = land_dao::settings::list_maps(keys).await?;
    info!("success, count:{}", settings.len());
    Ok((StatusCode::OK, Json(settings)))
}
