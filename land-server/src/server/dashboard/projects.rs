use super::auth::SessionUser;
use crate::server::{
    dashboard::vars::ProjectVar,
    redirect_response,
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars, ServerError,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use axum::{Form, Json};
use axum_csrf::CsrfToken;
use base64::{engine::general_purpose, Engine};
use land_dao::projects::{Language, ProjectStatus};
use serde::Deserialize;
use tracing::info;

/// index is a handler for GET /projects
pub async fn index(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        projects: Vec<ProjectVar>,
    }

    // list all projects
    let projects_data = land_dao::projects::list_by_user_id(user.id, None, 99).await?;
    info!("List projects: {}, acc: {}", projects_data.len(), user.uuid);
    let mut projects = ProjectVar::from_models_vec(projects_data).await?;
    let project_ids = projects.iter().map(|p| p.id).collect::<Vec<i32>>();
    let summary_traffics = land_dao::traffic::summary_projects_traffic(project_ids).await?;
    for p in projects.iter_mut() {
        if let Some(traffic) = summary_traffics.get(&p.id) {
            p.traffic = Some(traffic.clone());
        }
    }
    Ok(RenderHtmlMinified(
        "projects.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Projects", "projects"),
            user,
            projects,
        },
    ))
}

/// new is a handler for GET /new
pub async fn new(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
    }
    RenderHtmlMinified(
        "project-new.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Create a Project", "projects"),
            user,
        },
    )
}

// static http-javascript template content
static HTTP_JAVASCRIPT_TEMPLATE: &str = r#"ZXhwb3J0IGRlZmF1bHQgewogICAgYXN5bmMgZmV0Y2gocmVxdWVzdCkgewogICAgICAgIHJldHVybiBuZXcgUmVzcG9uc2UoYEhlbGxvLCBSdW50aW1lLmxhbmQgSlMgU0RLYCk7CiAgICB9Cn0="#;
static HTTP_JAVASCRIPT_DESCRIPTION: &str =
    "a simple HTTP router that shows Hello World written in JavaScript";

/// new_playground is a handler for GET /playground/new/:template
pub async fn new_playground(
    Extension(user): Extension<SessionUser>,
    Path(template): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let tpl = TemplateVar::from(template);
    if tpl.is_none() {
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Template not found",
        ));
    }
    let tpl = tpl.unwrap();
    let p = land_dao::projects::create_project_with_playground(
        user.id,
        tpl.language,
        tpl.description,
        tpl.content,
    )
    .await?;
    let dp = land_dao::deployment::create(user.id, user.uuid, p.id, p.uuid, p.prod_domain).await?;
    info!(
        "New playground and project, name: {}, dp: {}",
        p.name, dp.id
    );
    Ok(redirect_response(
        format!("/playground/{}", p.name).as_str(),
    ))
}

struct TemplateVar {
    description: String,
    language: Language,
    content: String,
}

impl TemplateVar {
    pub fn from(template: String) -> Option<Self> {
        if template.eq("hello-world-javascript") {
            return Some(TemplateVar {
                description: HTTP_JAVASCRIPT_DESCRIPTION.to_string(),
                language: Language::JavaScript,
                content: String::from_utf8_lossy(
                    &general_purpose::STANDARD
                        .decode(HTTP_JAVASCRIPT_TEMPLATE)
                        .unwrap(),
                )
                .to_string(),
            });
        }
        None
    }
}

/// show_playground is a handler for GET /playground/:name
pub async fn show_playground(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
    }
    let (p, py) = land_dao::projects::get_project_by_name_with_playground(name, user.id).await?;
    let project = ProjectVar::new(&p, py.as_ref()).await?;
    let title = format!("Playground - {}", project.name);
    Ok(RenderHtmlMinified(
        "playground.hbs",
        engine,
        IndexVars {
            page: PageVars::new(&title, "playground"),
            user,
            project,
        },
    ))
}

#[derive(Deserialize)]
pub struct PlaygroundForm {
    pub source: String,
}

/// save_playground is a handler for POST /playground/:name
pub async fn save_playground(
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
    Form(form): Form<PlaygroundForm>,
) -> Result<impl IntoResponse, ServerError> {
    let (p, py) = land_dao::projects::get_project_by_name_with_playground(name, user.id).await?;
    if py.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Playground not found",
        ));
    }
    // if project is deploying, show error
    if land_dao::deployment::is_deploying(p.id).await? {
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Project is deploying, please wait",
        ));
    }
    // update playground
    land_dao::projects::update_playground(p.id, user.id, form.source, &py.unwrap()).await?;
    // create deployment task, waiting to handle
    let dp = land_dao::deployment::create(user.id, user.uuid, p.id, p.uuid, p.prod_domain).await?;
    info!("Deployment task created: {:?}", dp);
    Ok(StatusCode::OK)
}

/// single is a handler for GET /projects/:name
pub async fn single(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
    }
    let (p, py) = land_dao::projects::get_project_by_name_with_playground(name, user.id).await?;
    let project = ProjectVar::new(&p, py.as_ref()).await?;

    let title = format!("Project - {}", project.name);
    Ok(RenderHtmlMinified(
        "project-single.hbs",
        engine,
        IndexVars {
            page: PageVars::new(&title, "project-dashboard"),
            user,
            project,
        },
    ))
}

// settings is a handler for GET /projects/:name/settings
pub async fn settings(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
        csrf: String,
    }
    let csrf = csrf_layer.authenticity_token()?;
    let p = land_dao::projects::get_by_name(name, Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let p = p.unwrap();
    let project = ProjectVar::new(&p, None).await?;
    let title = format!("Settings - {}", project.name);
    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "project-settings.hbs",
            engine,
            IndexVars {
                page: PageVars::new(&title, "project-settings"),
                user,
                project,
                csrf,
            },
        )
        .into_response(),
    ))
}

#[derive(Deserialize)]
pub struct UpdateNameForm {
    pub name: String,
    pub desc: String,
    pub csrf: String,
}

/// update_name is a handler for POST /projects/:name/settings
pub async fn update_name(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Path(name): Path<String>,
    Form(form): Form<UpdateNameForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let redirect_url = format!("/projects/{}/settings", form.name);
    // check if the project exists
    let p = land_dao::projects::get_by_name(name.clone(), Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let p = p.unwrap();
    if p.status == ProjectStatus::Disabled.to_string() {
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Project is disabled",
        ));
    }
    info!("Project rename, from: {}, to: {}", name, form.name);
    land_dao::projects::update_name(p.id, form.name, form.desc).await?;
    land_dao::deployment::create(user.id, user.uuid, p.id, p.uuid, p.prod_domain).await?;
    Ok(redirect_response(&redirect_url))
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub name: String,
    pub csrf: String,
}

/// delete is a handler for DELETE /projects/:name
pub async fn delete(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Path(name): Path<String>,
    Form(form): Form<DeleteForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let redirect_url = "/projects";
    // check if the project exists
    let p = land_dao::projects::get_by_name(name.clone(), Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let p = p.unwrap();
    if p.name != form.name {
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Project name mismatch",
        ));
    }
    info!("Project delete: {}", name);
    land_dao::projects::delete(p.id, name).await?;
    Ok(redirect_response(redirect_url))
}

// traffic is a handler for GET /projects/:name/traffic
pub async fn traffic(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        project: ProjectVar,
    }
    let p = land_dao::projects::get_by_name(name, Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let p = p.unwrap();
    let project = ProjectVar::new(&p, None).await?;
    let title = format!("Traffic - {}", project.name);
    Ok(RenderHtmlMinified(
        "project-traffic.hbs",
        engine,
        IndexVars {
            page: PageVars::new(&title, "project-traffic"),
            user,
            project,
        },
    ))
}

/// check_deploy is a handler for GET /projects/:name/check-deploy
pub async fn check_deploy(
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let p = land_dao::projects::get_by_name(name, Some(user.id)).await?;
    if p.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let p = p.unwrap();
    let dp = land_dao::deployment::get_last_by_project(p.id).await?;
    if dp.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Deployment not found",
        ));
    }
    Ok(Json(dp.unwrap()))
}
