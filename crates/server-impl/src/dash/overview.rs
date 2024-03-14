use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use chrono::NaiveDateTime;
use land_dao::{
    project::{self, CreatedBy},
    settings,
};
use serde::Serialize;
use tracing::info;

use super::auth::SessionUser;
use crate::{tpls::TemplateEngine, PageVars, ServerError};

#[derive(Debug, Serialize)]
pub struct ProjectVar {
    pub name: String,
    pub domain: String,
    pub domain_full: String,
    pub description: String,
    pub url: String,
    pub language: String,
    pub created_by: String,
    pub updated_at: NaiveDateTime,
    pub source: String,
    pub is_editable: bool,
    pub is_blank: bool,
}

impl ProjectVar {
    pub async fn from_models_vec(
        projects: Vec<land_dao::models::project::Model>,
    ) -> anyhow::Result<Vec<ProjectVar>> {
        let (domain, protocol) = settings::get_domain_settings().await?;
        Ok(projects
            .into_iter()
            .map(|p| ProjectVar {
                name: p.name.clone(),
                domain: p.domain.clone(),
                domain_full: format!("{}.{}", p.domain, domain),
                url: format!("{}://{}.{}", protocol, p.domain, domain),
                language: p.language,
                is_editable: p.created_by == CreatedBy::Playground.to_string(),
                is_blank: p.created_by == CreatedBy::Blank.to_string(),
                created_by: p.created_by,
                updated_at: p.updated_at,
                description: p.description,
                source: String::new(), // for list show, source is not needed
            })
            .collect())
    }
    pub async fn new(
        project: &land_dao::models::project::Model,
        playground: Option<&land_dao::models::playground::Model>,
    ) -> anyhow::Result<Self> {
        let (domain, protocol) = settings::get_domain_settings().await?;
        let mut var = ProjectVar {
            name: project.name.clone(),
            domain: project.domain.clone(),
            domain_full: format!("{}.{}", project.domain, domain),
            url: format!("{}://{}.{}", protocol, project.domain, domain),
            language: project.language.clone(),
            updated_at: project.updated_at,
            description: project.description.clone(),
            source: String::new(),
            created_by: project.created_by.clone(),
            is_editable: project.created_by == CreatedBy::Playground.to_string(),
            is_blank: project.created_by == CreatedBy::Blank.to_string(),
        };
        if let Some(playground) = playground {
            var.source = playground.source.clone();
        }
        Ok(var)
    }
}

/// index is a handler for GET /overview
pub async fn index(
    Extension(user): Extension<SessionUser>,
    engine: TemplateEngine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        page: PageVars,
        user: SessionUser,
        projects: Vec<ProjectVar>,
    }

    // show two lines, 1-2-3 in line, 4-5-view_all in next line
    let projects_data = project::list_by_user_id(user.id, None, 5).await?;
    info!("Overview projects: {}", projects_data.len());
    let projects = ProjectVar::from_models_vec(projects_data).await?;

    Ok(RenderHtml(
        "overview.hbs",
        engine,
        Vars {
            page: PageVars::new("Overview", "/overview", "overview"),
            user,
            projects,
        },
    ))
}
