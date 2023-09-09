use super::{auth::CurrentUser, params, AppError};
use axum::{extract::Path, http::StatusCode, Extension, Json};
use tracing::info;

fn is_admin(user: &CurrentUser) -> Result<(), AppError> {
    if !user.is_admin() {
        return Err(AppError(
            anyhow::anyhow!("permission denied"),
            StatusCode::FORBIDDEN,
        ));
    }
    Ok(())
}

#[tracing::instrument(name = "[ops_list_projects]", skip_all)]
pub async fn list_projects(
    Extension(current_user): Extension<CurrentUser>,
    Path(page): Path<u64>,
) -> Result<(StatusCode, Json<params::ProjectPagination>), AppError> {
    is_admin(&current_user)?;
    let current_page = if page > 0 { page - 1 } else { 0 };
    let (projects, total_page, total_items) =
        land_dao::project::get_pagination(current_page, 10).await?;
    let values = params::ProjectResponse::from_models(&projects).await;
    info!(
        "success, total_items:{}, total_pages:{}, current_page:{}",
        total_items, total_page, current_page
    );
    let result = params::ProjectPagination {
        projects: values,
        total_page,
        total_items,
    };
    Ok((StatusCode::OK, Json(result)))
}
