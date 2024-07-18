use std::str::FromStr;

use super::{redirect, ServerError};
use crate::{
    dash::{error_html, notfound_html},
    templates::Engine,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form};
use axum_htmx::HxRedirect;
use axum_template::RenderHtml;
use land_core::examples::{self, Item};
use land_dao::{projects, settings};
use land_vars::{AuthUser, BreadCrumbKey, Page, Project};
use serde::Serialize;
use tracing::{info, warn};

/// index is handler for projects index page, /projects
pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub projects: Vec<Project>,
    }
    let projects_data = land_dao::projects::list(Some(user.id), None, 1, 50).await?;
    Ok(RenderHtml(
        "projects.hbs",
        engine,
        Vars {
            page: Page::new("Projects", BreadCrumbKey::Projects, Some(user)),
            projects: Project::new_from_models(projects_data).await?,
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
        pub project: Project,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_html(engine, &msg, user).into_response());
    }
    let project = Project::new_with_source(&project.unwrap()).await?;
    Ok(RenderHtml(
        "project-single.hbs",
        engine,
        Vars {
            page: Page::new(&name, BreadCrumbKey::ProjectSingle, Some(user)),
            project_name: name,
            project,
        },
    )
    .into_response())
}

/// traffic is handler for projects traffic page, /projects/:name/traffic
pub async fn traffic(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub project_name: String,
        pub project: Project,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_html(engine, &msg, user).into_response());
    }
    let project = Project::new_with_source(&project.unwrap()).await?;
    Ok(RenderHtml(
        "project-traffic.hbs",
        engine,
        Vars {
            page: Page::new(&name, BreadCrumbKey::ProjectTraffic, Some(user)),
            project_name: name,
            project,
        },
    )
    .into_response())
}

/// settings is handler for projects settings page, /projects/:name/settings
pub async fn settings(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub project_name: String,
        pub project: Project,
        pub domain: String,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_html(engine, &msg, user).into_response());
    }
    let domain_settings = settings::get_domain_settings().await?;
    let project = Project::new_with_source(&project.unwrap()).await?;
    Ok(RenderHtml(
        "project-settings.hbs",
        engine,
        Vars {
            page: Page::new(&name, BreadCrumbKey::ProjectSettings, Some(user)),
            project_name: name,
            project,
            domain: domain_settings.domain_suffix,
        },
    )
    .into_response())
}

#[derive(serde::Deserialize, Debug)]
pub struct SettingsForm {
    pub name: String,
    pub description: String,
}

pub async fn handle_settings(
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
    Form(f): Form<SettingsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        return Ok(error_html("Project not found").into_response());
    }
    if name != f.name && !projects::is_unique_name(&f.name).await? {
        warn!(
            owner_id = user.id,
            project_name = f.name,
            "Project name already exists",
        );
        return Ok(error_html("Project name already exists").into_response());
    }
    let project = project.unwrap();
    projects::update_names(project.id, &f.name, &f.description).await?;
    info!(
        owner_id = user.id,
        project_old_name = name,
        project_new_name = f.name,
        "Update project names",
    );
    let uri = axum::http::Uri::from_str(format!("/projects/{}/settings", f.name).as_str())?;
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}
