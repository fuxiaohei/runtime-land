use crate::{dash::ServerError, templates::Engine};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use land_dao::{projects, users};
use land_vars::{AuthUser, BreadCrumbKey, Page, Pagination, Project};
use serde::Serialize;

pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub projects: Vec<Project>,
        pub pagination: Pagination,
    }

    let (project_models, pager) = projects::list(None, None, 1, 20).await?;
    let mut projects = Project::new_from_models(project_models).await?;
    let pagination = Pagination::new(
        1,
        20,
        pager.number_of_pages,
        pager.number_of_items,
        "/admin/projects",
    );

    // read owners to fill
    let mut owner_ids = vec![];
    for project in &projects {
        owner_ids.push(project.owner_id);
    }
    let owners = users::find_by_ids(owner_ids).await?;
    for project in &mut projects {
        let owner = owners.get(&project.owner_id);
        if let Some(owner) = owner {
            project.owner = Some(AuthUser::new(owner));
        }
    }

    Ok(RenderHtml(
        "admin/projects.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin", BreadCrumbKey::AdminProjects, Some(user)),
            projects,
            pagination,
        },
    ))
}
