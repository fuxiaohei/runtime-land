use land_dao::{models, projects::ProjectCreatedBy, settings, DateTimeUTC};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProjectVar {
    pub id: i32,
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
    pub updated_at: DateTimeUTC,
    pub source: String,
    pub is_editable: bool,
    pub deploy_status: String,
    pub status: String,
}

impl ProjectVar {
    pub async fn from_models_vec(
        projects: Vec<models::project::Model>,
    ) -> anyhow::Result<Vec<ProjectVar>> {
        let (domain, protocol) = settings::get_domain_settings().await?;
        Ok(projects
            .into_iter()
            .map(|p| ProjectVar {
                id: p.id,
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
                updated_at: p.updated_at.and_utc(),
                description: p.description,
                source: String::new(), // for list show, source is not needed
                deploy_status: p.deploy_status,
                status: p.status,
            })
            .collect())
    }
    pub async fn new(
        project: &land_dao::models::project::Model,
        playground: Option<&land_dao::models::playground::Model>,
    ) -> anyhow::Result<Self> {
        let (domain, protocol) = settings::get_domain_settings().await?;
        let mut var = ProjectVar {
            id: project.id,
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
            updated_at: project.updated_at.and_utc(),
            description: project.description.clone(),
            source: String::new(),
            created_by: project.created_by.clone(),
            is_editable: project.created_by == ProjectCreatedBy::Playground.to_string(),
            deploy_status: project.deploy_status.clone(),
            status: project.status.clone(),
        };
        if let Some(playground) = playground {
            var.source = playground.source.clone();
        }
        Ok(var)
    }
}

#[derive(Serialize)]
pub struct TokenVar {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub is_new: bool,
    pub updated_at: DateTimeUTC,
}

#[derive(Serialize)]
pub struct WorkerVar {
    pub id: i32,
    pub ip: String,
    pub hostname: String,
    pub updated_at: DateTimeUTC,
    pub status: String,
}

impl WorkerVar {
    pub fn from_models_vec(workers: Vec<models::worker::Model>) -> Vec<WorkerVar> {
        workers
            .into_iter()
            .map(|w| WorkerVar {
                id: w.id,
                ip: w.ip,
                hostname: w.hostname,
                updated_at: w.updated_at.and_utc(),
                status: w.status.to_string(),
            })
            .collect()
    }
}

#[derive(Serialize)]
pub struct PaginationVarItem {
    pub link: String,
    pub current: bool,
    pub page: u64,
}

#[derive(Serialize)]
pub struct PaginationVar {
    pub current: u64,
    pub count: u64,
    pub total: u64,
    pub items: Vec<PaginationVarItem>,
}

impl PaginationVar {
    pub fn new(current: u64, size: u64, count: u64, total: u64, link: &str) -> PaginationVar {
        let mut items = vec![];
        for i in 1..=total {
            items.push(PaginationVarItem {
                link: format!("{}?page={}&size={}", link, i, size),
                current: i == current,
                page: i,
            });
        }
        PaginationVar {
            current,
            count,
            total,
            items,
        }
    }
}
