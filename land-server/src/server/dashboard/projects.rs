use super::auth::SessionUser;
use crate::server::{
    redirect_response,
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars, ServerError,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use base64::{engine::general_purpose, Engine};
use land_dao::projects::Language;

/// index is a handler for GET /projects
pub async fn index(engine: TemplateEngine) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
    }
    // redirect to /overview
    RenderHtmlMinified(
        "projects.hbs",
        engine,
        IndexVars {
            page: PageVars::new("Projects", "projects"),
        },
    )
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
