use anyhow::Result;
use land_dao::deployment::{DeployStatus, DeploymentStatus};
use land_dao::models::{deployment, deployment_task};
use land_dao::{projects, user, worker};
use serde::Serialize;
use strum::IntoEnumIterator;

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
    pub user_name: String,
    pub user_email: String,
    pub user_oauth: String,
}

impl DeployVars {
    pub async fn from_models(dps: Vec<deployment::Model>) -> Result<Vec<DeployVars>> {
        let mut vars = vec![];
        let mut project_ids = vec![];
        for dp in &dps {
            project_ids.push(dp.project_id);
        }
        let projects = projects::list_by_ids(project_ids).await?;

        let mut user_ids = vec![];
        for dp in &dps {
            user_ids.push(dp.user_id);
        }
        // unique user_ids
        user_ids.sort();
        user_ids.dedup();
        let users = land_dao::user::list_infos(user_ids).await?;

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
                user_name: "".to_string(),
                user_email: "".to_string(),
                user_oauth: "".to_string(),
            };
            if let Some(project) = projects.get(&dp.project_id) {
                v.language.clone_from(&project.language);
                v.created_by.clone_from(&project.created_by);
            }
            if let Some(user) = users.get(&dp.user_id) {
                v.user_name.clone_from(&user.nick_name);
                v.user_email.clone_from(&user.email);
                v.user_oauth.clone_from(&user.origin_provider);
            }
            vars.push(v);
        }
        Ok(vars)
    }

    pub async fn from_model(dp: deployment::Model) -> Result<DeployVars> {
        let project = projects::get_by_id(dp.project_id, None).await?;
        let user = user::get_info_by_id(dp.user_id, None).await?;
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
            user_name: "".to_string(),
            user_email: "".to_string(),
            user_oauth: "".to_string(),
        };
        if let Some(project) = project {
            v.language.clone_from(&project.language);
            v.created_by.clone_from(&project.created_by);
        }
        if let Some(user) = user {
            v.user_name.clone_from(&user.nick_name);
            v.user_email.clone_from(&user.email);
            v.user_oauth.clone_from(&user.origin_provider);
        }
        Ok(v)
    }
}

#[derive(Serialize)]
pub struct DeployDetailVars {
    pub id: i32,
    pub task_id: String,
    pub deploy_id: i32,
    pub worker_id: i32,
    pub worker_ip: String,
    pub deploy_status: String,
    pub deploy_message: String,
    pub created_at: String,
    pub updated_at: String,
}

impl DeployDetailVars {
    pub async fn from_models(tasks: Vec<deployment_task::Model>) -> Result<Vec<DeployDetailVars>> {
        let mut vars = vec![];
        let mut worker_ids = vec![];
        for task in &tasks {
            worker_ids.push(task.worker_id);
        }
        worker_ids.sort();
        worker_ids.dedup();

        let workers = worker::list_by_ids(worker_ids).await?;
        for task in tasks {
            vars.push(DeployDetailVars {
                id: task.id,
                task_id: task.task_id,
                deploy_id: task.deployment_id,
                worker_id: task.worker_id,
                worker_ip: "".to_string(),
                deploy_status: task.deploy_status,
                deploy_message: task.deploy_message,
                created_at: task.created_at.to_string(),
                updated_at: task.updated_at.to_string(),
            });
        }
        if !workers.is_empty() {
            for v in vars.iter_mut() {
                if let Some(worker) = workers.get(&v.worker_id) {
                    v.worker_ip.clone_from(&worker.ip);
                }
            }
        }
        Ok(vars)
    }
}

#[derive(Serialize, Debug)]
pub struct DeployStatusVars {
    pub value: String,
    pub is_selected: bool,
}

impl DeployStatusVars {
    pub fn new_list(selected: &str) -> Vec<DeployStatusVars> {
        let mut vars = vec![];
        vars.push(DeployStatusVars {
            value: "all".to_string(),
            is_selected: selected.is_empty(),
        });
        for status in DeployStatus::iter() {
            vars.push(DeployStatusVars {
                value: status.to_string(),
                is_selected: status.to_string() == selected,
            });
        }
        vars
    }
}

#[derive(Serialize, Debug)]
pub struct DeployCommonStatusVars {
    pub value: String,
    pub label: String,
    pub is_selected: bool,
}

impl DeployCommonStatusVars {
    pub fn new_list(selected: &str) -> Vec<DeployCommonStatusVars> {
        let mut vars = vec![];
        vars.push(DeployCommonStatusVars {
            value: "".to_string(),
            label: "Available".to_string(),
            is_selected: selected.is_empty(),
        });
        for status in DeploymentStatus::iter() {
            vars.push(DeployCommonStatusVars {
                value: status.to_string(),
                label: status.to_string(),
                is_selected: status.to_string() == selected,
            });
        }
        vars
    }
}
