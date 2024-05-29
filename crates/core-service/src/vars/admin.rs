use anyhow::Result;
use land_dao::models::deployment;
use land_dao::projects;
use serde::Serialize;

/// DashboardVars is the vars for admin dashboard page
#[derive(Serialize)]
pub struct DashboardVars {
    pub user_count: u64,
    pub project_count: u64,
    pub total_requests: i32,
    pub total_bytes: i32,
}

impl DashboardVars {
    pub async fn new() -> Result<Self> {
        let user_count = land_dao::user::count().await?;
        let project_count = land_dao::projects::count().await?;
        let traffic_summary = land_dao::traffic::get_current_total().await?;
        Ok(Self {
            user_count,
            project_count,
            total_requests: traffic_summary.as_ref().map_or(0, |s| s.requests),
            total_bytes: traffic_summary.as_ref().map_or(0, |s| s.transferred_bytes),
        })
    }
}

#[derive(Serialize)]
pub struct DeployVars {
    pub id: i32,
    pub domain: String,
    pub language: String,
    pub created_by: String,
    pub size: i32,
    pub deploy_status: String,
    pub deploy_message: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl DeployVars {
    pub async fn from_models(dps: Vec<deployment::Model>) -> Result<Vec<DeployVars>> {
        let mut vars = vec![];
        let mut project_ids = vec![];
        for dp in &dps {
            project_ids.push(dp.project_id);
        }
        let projects = projects::list_by_ids(project_ids).await?;
        for dp in dps {
            let mut v = DeployVars {
                id: dp.id,
                domain: dp.domain,
                language: "".to_string(),
                created_by: "".to_string(),
                size: dp.storage_size,
                deploy_status: dp.deploy_status,
                deploy_message: dp.deploy_message,
                status: dp.status,
                created_at: dp.created_at.to_string(),
                updated_at: dp.updated_at.to_string(),
            };
            if let Some(project) = projects.get(&dp.project_id) {
                v.language.clone_from(&project.language);
                v.created_by.clone_from(&project.created_by);
            }
            vars.push(v);
        }
        Ok(vars)
    }
}
