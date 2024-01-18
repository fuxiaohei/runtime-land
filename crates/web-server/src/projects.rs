use crate::{sign::SessionUser, AppError, PageVars, RenderEngine};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use chrono::{DateTime, Utc};
use land_dblayer::{project, settings};
use serde::{Deserialize, Serialize};

/// index is the handler for /projects
pub async fn index(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> Result<impl IntoResponse, AppError> {
    let projects = project::list_by_owner(user.id).await?;

    #[derive(Debug, Serialize, Deserialize)]
    struct ProjectVars {
        pub name: String,
        pub language: String,
        pub prod_domain: String,
        pub prod_url: String,
        pub created_by: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub uuid: String,
    }

    let (domain_suffix, prod_protocol) = settings::get_domain_settings().await?;

    let project_vars: Vec<ProjectVars> = projects
        .iter()
        .map(|p| {
            let prod_url = if p.prod_domain.is_empty() {
                String::new()
            } else {
                format!("{}://{}.{}", prod_protocol, p.prod_domain, domain_suffix)
            };
            let prod_domain = if p.prod_domain.is_empty() {
                String::new()
            } else {
                format!("{}.{}", p.prod_domain, domain_suffix)
            };
            ProjectVars {
                name: p.name.clone(),
                language: p.language.clone(),
                prod_domain,
                prod_url,
                created_by: p.created_by.clone(),
                created_at: p.created_at,
                updated_at: p.updated_at,
                uuid: p.uuid.clone(),
            }
        })
        .collect::<Vec<ProjectVars>>();

    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
        pub projects: Vec<ProjectVars>,
        pub projects_count: usize,
    }

    Ok(RenderHtml(
        "projects.hbs",
        engine,
        Vars {
            page: PageVars::new("Projects", "/projects"),
            user,
            projects_count: projects.len(),
            projects: project_vars,
        },
    ))
}
