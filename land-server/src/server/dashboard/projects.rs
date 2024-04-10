use super::auth::SessionUser;
use crate::server::{
    dashboard::vars::ProjectVar,
    redirect_response,
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars, ServerError,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use base64::{engine::general_purpose, Engine};
use land_dao::projects::Language;
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
    info!("List projects: {}", projects_data.len());
    let projects = ProjectVar::from_models_vec(projects_data).await?;
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
pub async fn new(engine: TemplateEngine) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
    }
    RenderHtmlMinified(
        "project-new.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Create a Project", "projects"),
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
    let project_name = land_dao::projects::create_project_with_playground(
        user.id,
        tpl.language,
        tpl.description,
        tpl.content,
    )
    .await?;
    Ok(redirect_response(
        format!("/playground/{}", project_name).as_str(),
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
    engine: TemplateEngine,
    Path(_name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
    }
    // redirect to /overview
    Ok(RenderHtmlMinified(
        "playground.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Playground", "playground"),
        },
    ))
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

    let title = format!("{} - Project", project.name);
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
    let title = format!("{} - Settings", project.name);
    Ok(RenderHtmlMinified(
        "project-settings.hbs",
        engine,
        IndexVars {
            page: PageVars::new(&title, "project-settings"),
            user,
            project,
        },
    ))
}
