use axum::extract::Query;
use axum::Form;
use axum::{extract::Path, response::IntoResponse, Extension};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use land_dao::{playground, project, settings};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::auth::SessionUser;
use super::overview::ProjectVar;
use crate::{redirect_response, tpls::TemplateEngine, PageVars, ServerError};

/// new is a handler for GET /new
pub async fn new(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        csrf_token: String,
        random_project_name: String,
    }
    let csrf_token = csrf.authenticity_token()?;
    let random_project_name = project::random_name();
    Ok((
        csrf,
        RenderHtml(
            "project-new.hbs",
            engine,
            Vars {
                page: PageVars::new("Create a project", "/new", "projects"),
                user,
                csrf_token,
                random_project_name,
            },
        ),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct SearchQuery {
    search: Option<String>,
}

/// new is a handler for GET /new
pub async fn index(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Query(q): Query<SearchQuery>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        csrf_token: String,
        projects: Vec<ProjectVar>,
        search: String,
    }
    let csrf_token = csrf.authenticity_token()?;
    let projects_data = project::list_by_user_id(user.id, q.search.clone(), 999).await?;
    info!("Projects table: {}", projects_data.len());
    let projects = ProjectVar::from_models_vec(projects_data).await?;
    Ok((
        csrf,
        RenderHtml(
            "project-list.hbs",
            engine,
            Vars {
                page: PageVars::new("Projects", "/projects", "projects"),
                user,
                csrf_token,
                projects,
                search: q.search.unwrap_or_default(),
            },
        ),
    )
        .into_response())
}

/// new_blank is a handler for GET /new/blank
pub async fn new_blank(
    Extension(user): Extension<SessionUser>,
) -> Result<impl IntoResponse, ServerError> {
    let project = project::create_blank(user.id).await?;
    info!("Create blank project: {:?}", project);
    Ok(redirect_response(
        format!("/projects/{}", project.name).as_str(),
    ))
}

/// single is the handler for /projects/:name
pub async fn single(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
    }
    let project_value = project::get_by_name(name, Some(user.id)).await?;
    if project_value.is_none() {
        return Err(ServerError::not_found("Project not found"));
    }
    let project_value = project_value.unwrap();

    // if the project is created by playground, then get the source from playground
    let mut playgrond: Option<land_dao::models::playground::Model> = None;
    if project_value.created_by == land_dao::project::CreatedBy::Playground.to_string() {
        playgrond = playground::get_by_project(user.id, project_value.id).await?;
    }
    info!("Project single: {:?}", project_value);
    let project = ProjectVar::new(&project_value, playgrond.as_ref()).await?;

    let title = format!("{} - Project", project.name);
    let base_uri = format!("/projects/{}", project.name);
    Ok(RenderHtml(
        "project-single.hbs",
        engine,
        Vars {
            page: PageVars::new(&title, &base_uri, "projects"),
            user,
            project,
        },
    ))
}

/// traffic is the handler for /projects/:name/traffic
pub async fn traffic(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
    }
    let project_value = project::get_by_name(name, Some(user.id)).await?;
    if project_value.is_none() {
        return Err(ServerError::not_found("Project not found"));
    }
    let project_value = project_value.unwrap();
    info!("Project traffic: {:?}", project_value);
    let project = ProjectVar::new(&project_value, None).await?;

    let title = format!("{} - Project", project.name);
    let base_uri = format!("/projects/{}/traffic", project.name);
    Ok(RenderHtml(
        "project-traffic.hbs",
        engine,
        Vars {
            page: PageVars::new(&title, &base_uri, "projects"),
            user,
            project,
        },
    ))
}

/// settings is the handler for /projects/:name/settings
pub async fn settings(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
        domain: String,
        csrf_token: String,
    }
    let csrf_token = csrf.authenticity_token()?;

    let project_value = project::get_by_name(name, Some(user.id)).await?;
    if project_value.is_none() {
        return Err(ServerError::not_found("Project not found"));
    }
    let project_value = project_value.unwrap();
    info!("Project settings: {:?}", project_value);
    let project = ProjectVar::new(&project_value, None).await?;

    let (domain, _) = settings::get_domain_settings().await?;
    let title = format!("{} - Project", project.name);
    let base_uri = format!("/projects/{}/settings", project.name);
    Ok((
        csrf,
        RenderHtml(
            "project-settings.hbs",
            engine,
            Vars {
                page: PageVars::new(&title, &base_uri, "projects"),
                user,
                project,
                domain,
                csrf_token,
            },
        ),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct SettingsNameForm {
    domain: String,
    csrf: String,
}

/// settings_post_name is the handler for POST /projects/:name/settings
pub async fn settings_post_domain(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
    Form(form): Form<SettingsNameForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf.verify(&form.csrf)?;
    let project_value = project::get_by_name(name, Some(user.id)).await?;
    if project_value.is_none() {
        return Err(ServerError::not_found("Project not found"));
    }
    let project_value = project_value.unwrap();
    project::update_name(project_value.id, form.domain.clone()).await?;
    let url = format!("/projects/{}", form.domain);
    Ok(redirect_response(&url))
}

#[derive(Deserialize)]
pub struct DeleteForm {
    name: String,
    csrf: String,
}

/// post_delete is the handler for POST /projects/:name/delete
pub async fn post_delete(
    csrf: CsrfToken,
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
    Form(form): Form<DeleteForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf.verify(&form.csrf)?;
    if name != form.name {
        return Err(ServerError::bad_request("Project name not matched"));
    }
    let project = project::get_by_name(name, Some(user.id)).await?;
    if project.is_none() {
        return Err(ServerError::not_found("Project not found"));
    }
    info!("Delete project: {:?}", project);
    project::delete(user.id, project.unwrap().id).await?;
    Ok(redirect_response("/overview"))
}
