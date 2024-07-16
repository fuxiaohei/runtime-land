use super::{redirect, ServerError};
use crate::templates::Engine;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use axum_template::RenderHtml;
use land_core::examples::{self, Item};
use land_dao::projects;
use land_vars::{AuthUser, BreadCrumbKey, Page};
use serde::Serialize;
use tracing::info;

/// index is handler for projects index page, /projects
pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
    }
    Ok(RenderHtml(
        "projects.hbs",
        engine,
        Vars {
            page: Page::new("Projects", BreadCrumbKey::Projects, Some(user)),
        },
    ))
}

/// new is handler for projects new page, /new
pub async fn new(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub examples: Vec<Item>,
    }
    let examples = examples::defaults();
    Ok(RenderHtml(
        "project-new.hbs",
        engine,
        Vars {
            page: Page::new("New Project", BreadCrumbKey::ProjectNew, Some(user)),
            examples,
        },
    ))
}

/// handle_new is handler for projects new page, /new/:name
pub async fn handle_new(
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let example = examples::get(&name);
    if example.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Template not found",
        ));
    }
    let example = example.unwrap();
    let source = example.get_source()?;
    if source.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Template source not found",
        ));
    }
    let (project, playground) = projects::create_with_playground(
        user.id,
        example.lang.parse()?,
        example.description,
        source.unwrap(),
    )
    .await?;
    info!(
        owner_id = user.id,
        project_name = project.name,
        playground_id = playground.id,
        tpl_name = name,
        "Create new project",
    );
    Ok(redirect(format!("/projects/{}", project.name).as_str()))
}

/// single is handler for projects single page, /projects/:name
pub async fn single(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub project_name: String,
    }
    Ok(RenderHtml(
        "project-single.hbs",
        engine,
        Vars {
            page: Page::new(&name, BreadCrumbKey::ProjectSingle, Some(user)),
            project_name: name,
        },
    ))
}
