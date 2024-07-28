use crate::{dash::ServerError, templates::Engine};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use land_dao::{projects, users};
use land_vars::{AuthUser, BreadCrumbKey, Page, Pagination};
use serde::Serialize;

/// index is route of users manage page, /admin/users/
pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub users: Vec<AuthUser>,
        pub pagination: Pagination,
    }
    let (user_models, pager) = users::list(None, 1, 50).await?;
    let mut users: Vec<_> = user_models.iter().map(AuthUser::new).collect();
    let pagination = Pagination::new(
        1,
        20,
        pager.number_of_pages,
        pager.number_of_items,
        "/admin/users",
    );

    let user_ids = user_models.iter().map(|u| u.id).collect::<Vec<_>>();
    let projects_counts = projects::count_by_users(user_ids).await?;
    for (user_id, count) in projects_counts {
        users
            .iter_mut()
            .find(|u| u.id == user_id)
            .unwrap()
            .projects_count = Some(count);
    }

    Ok(RenderHtml(
        "admin/users.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin", BreadCrumbKey::AdminUsers, Some(user)),
            users,
            pagination,
        },
    ))
}
