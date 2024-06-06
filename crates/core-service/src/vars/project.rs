use land_dao::projects::ProjectCreatedBy;
use land_dao::traffic::TrafficSummary;
use land_dao::{models, settings, DateTimeUTC};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProjectVar {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub user_email: String,
    pub user_nickname: String,
    pub user_id: i32,
    pub prod_domain: String,
    pub prod_domain_full: String,
    pub prod_domain_url: String,
    pub dev_domain: String,
    pub dev_domain_full: String,
    pub dev_domain_url: String,
    pub description: String,
    pub language: String,
    pub created_by: String,
    pub created_at: DateTimeUTC,
    pub updated_at: DateTimeUTC,
    pub source: String,
    pub is_editable: bool,
    pub deploy_status: String,
    pub status: String,
    pub traffic: Option<TrafficSummary>,
}

impl ProjectVar {
    pub async fn from_models_vec(
        projects: Vec<models::project::Model>,
    ) -> anyhow::Result<Vec<ProjectVar>> {
        let (domain, protocol, _) = settings::get_domain_settings().await?;
        Ok(projects
            .into_iter()
            .map(|p| ProjectVar {
                id: p.id,
                uuid: p.uuid.clone(),
                name: p.name.clone(),
                user_email: String::new(),
                user_nickname: String::new(),
                user_id: p.user_id,
                prod_domain: p.prod_domain.clone(),
                prod_domain_full: format!("{}.{}", p.prod_domain, domain),
                prod_domain_url: format!("{}://{}.{}", protocol, p.prod_domain, domain),
                dev_domain: p.dev_domain.clone(),
                dev_domain_full: format!("{}.{}", p.dev_domain, domain),
                dev_domain_url: format!("{}://{}.{}", protocol, p.dev_domain, domain),
                language: p.language,
                is_editable: p.created_by == ProjectCreatedBy::Playground.to_string(),
                created_by: p.created_by,
                created_at: p.created_at.and_utc(),
                updated_at: p.updated_at.and_utc(),
                description: p.description,
                source: String::new(), // for list show, source is not needed
                deploy_status: p.deploy_status,
                status: p.status,
                traffic: None,
            })
            .collect())
    }
    pub async fn new(
        project: &land_dao::models::project::Model,
        playground: Option<&land_dao::models::playground::Model>,
    ) -> anyhow::Result<Self> {
        let (domain, protocol, _) = settings::get_domain_settings().await?;
        let mut var = ProjectVar {
            id: project.id,
            uuid: project.uuid.clone(),
            name: project.name.clone(),
            user_email: String::new(),
            user_nickname: String::new(),
            user_id: project.user_id,
            prod_domain: project.prod_domain.clone(),
            prod_domain_full: format!("{}.{}", project.prod_domain, domain),
            prod_domain_url: format!("{}://{}.{}", protocol, project.prod_domain, domain),
            dev_domain: project.dev_domain.clone(),
            dev_domain_full: format!("{}.{}", project.dev_domain, domain),
            dev_domain_url: format!("{}://{}.{}", protocol, project.dev_domain, domain),
            language: project.language.clone(),
            created_at: project.created_at.and_utc(),
            updated_at: project.updated_at.and_utc(),
            description: project.description.clone(),
            source: String::new(),
            created_by: project.created_by.clone(),
            is_editable: project.created_by == ProjectCreatedBy::Playground.to_string(),
            deploy_status: project.deploy_status.clone(),
            status: project.status.clone(),
            traffic: None,
        };
        if let Some(playground) = playground {
            var.source.clone_from(&playground.source);
        }
        Ok(var)
    }
}
