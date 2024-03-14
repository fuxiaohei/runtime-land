use super::auth::SessionUser;
use crate::{redirect_response, tpls::TemplateEngine, PageVars, ServerError};
use axum::{extract::Path, response::IntoResponse, Extension, Form, Json};
use axum_template::RenderHtml;
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use land_dao::{
    deployment,
    project::{self, Language},
    settings,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct PlaygroundVar {
    pub name: String,
    pub domain: String,
    pub domain_full: String,
    pub description: String,
    pub url: String,
    pub language: String,
    pub source: String,
    pub updated_at: NaiveDateTime,
    pub deploy_status: String,
}

impl PlaygroundVar {
    pub async fn new(
        project: &land_dao::models::project::Model,
        playground: &land_dao::models::playground::Model,
    ) -> anyhow::Result<Self> {
        let (domain, protocol) = settings::get_domain_settings().await?;
        let var = PlaygroundVar {
            name: project.name.clone(),
            domain: project.domain.clone(),
            domain_full: format!("{}.{}", project.domain, domain),
            url: format!("{}://{}.{}", protocol, project.domain, domain),
            language: project.language.clone(),
            updated_at: playground.created_at,
            description: project.description.clone(),
            source: playground.source.clone(),
            deploy_status: String::new(),
        };
        Ok(var)
    }
}

/// index is the handler for GET /playground/:name
pub async fn index(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        project: PlaygroundVar,
    }

    let (p, py, dp) = match land_kernel::playground::get_by_project(name, user.id).await {
        Ok((p, py, dp)) => (p, py, dp),
        Err(e) => return Err(ServerError::not_found(&e.to_string())),
    };

    let mut pvar = PlaygroundVar::new(&p, &py).await?;
    if let Some(dp) = dp {
        pvar.deploy_status = dp.deploy_status;
    }

    let title = format!("{} - Playground", p.name);
    let base_uri = format!("/playground/{}", p.name);
    Ok(RenderHtml(
        "playground.hbs",
        engine,
        Vars {
            page: PageVars::new(&title, &base_uri, ""),
            user,
            project: pvar,
        },
    ))
}

#[derive(Debug, Deserialize)]
pub struct PlaygroundSaveRequest {
    pub source: String,
}

/// save is the handler for POST /playground/:name
pub async fn save(
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
    Form(payload): Form<PlaygroundSaveRequest>,
) -> Result<impl IntoResponse, ServerError> {
    land_kernel::playground::save_source(name, user.id, payload.source).await?;
    Ok("ok".into_response())
}

/// check is the handler for GET /playground/:name/check
pub async fn check(
    Extension(user): Extension<SessionUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let project = project::get_by_name(name, Some(user.id)).await?;
    if project.is_none() {
        return Err(ServerError::not_found("Project not found"));
    }
    let project = project.unwrap();
    let dp = deployment::get_by_project(user.id, project.id).await?;
    if dp.is_none() {
        return Err(ServerError::not_found("Deployment not found"));
    }
    let dp = dp.unwrap();
    Ok(Json(dp).into_response())
}

/// new is a handler for GET /new/playground/:template
pub async fn new(
    Extension(user): Extension<SessionUser>,
    Path(template): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let tplvar = TemplateVar::from(template);
    if tplvar.is_none() {
        return Err(ServerError::bad_request("Invalid template"));
    }
    let tplvar = tplvar.unwrap();
    let pname = land_kernel::playground::create(
        user.id,
        tplvar.language,
        tplvar.description,
        tplvar.content,
    )
    .await?;
    Ok(redirect_response(format!("/playground/{}", pname).as_str()))
}

// static http-javascript template content
static HTTP_JAVASCRIPT_TEMPLATE: &str = r#"ZXhwb3J0IGRlZmF1bHQgewogICAgYXN5bmMgZmV0Y2gocmVxdWVzdCkgewogICAgICAgIHJldHVybiBuZXcgUmVzcG9uc2UoYEhlbGxvLCBSdW50aW1lLmxhbmQgSlMgU0RLYCk7CiAgICB9Cn0="#;
static HTTP_JAVASCRIPT_DESCRIPTION: &str = "Simple http request handler with javascript";

struct TemplateVar {
    description: String,
    language: Language,
    content: String,
}

impl TemplateVar {
    pub fn from(template: String) -> Option<Self> {
        if template.eq("http-javascript") {
            return Some(TemplateVar {
                description: HTTP_JAVASCRIPT_DESCRIPTION.to_string(),
                language: Language::Js,
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
