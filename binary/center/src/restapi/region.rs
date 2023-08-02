use super::{auth::CurrentUser, params, AppError};
use axum::{http::StatusCode, Extension, Json};
use land_dao::user;
use tracing::info;

/// list_handler lists all regions
#[tracing::instrument(name = "[list_region]", skip_all)]
pub async fn list_handler(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, Json<Vec<params::RegionResponse>>), AppError> {
    if current_user.role != user::Role::Admin.to_string() {
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
            key: region.key,
            runtimes: region.runtimes,
            status: region.status,
        })
        .collect();
    info!("success, count:{}", values.len());
    Ok((StatusCode::OK, Json(values)))
}
