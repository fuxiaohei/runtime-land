use crate::db::DB;
use crate::models::deployment;
use crate::now_time;
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Spec {
    cpu_time_limit: Option<i32>,
    memory_limit: Option<i32>,
    wall_time_limit: Option<i32>,
    fetch_limit: Option<i32>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            cpu_time_limit: Some(100), // 100ms
            memory_limit: Some(128),   // 128MB
            wall_time_limit: Some(30), // 30 seconds
            fetch_limit: Some(5),      // send 5 requests
        }
    }
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeployStatus {
    Waiting,
    Compiling,
    Uploading,
    Deploying,
    Success,
    Failed,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeploymentStatus {
    Active,
    Deleted,
}

/// create a deployment
pub async fn create(
    user_id: i32,
    user_uuid: String,
    project_id: i32,
    project_uuid: String,
    domain: String,
) -> Result<deployment::Model> {
    let spec = Spec::default();
    let now = now_time();
    let model = deployment::Model {
        id: 0,
        user_id,
        user_uuid,
        project_id,
        project_uuid,
        task_id: uuid::Uuid::new_v4().to_string(),
        domain,
        storage_path: "".to_string(),
        storage_md5: "".to_string(),
        storage_size: 0,
        spec: serde_json::to_value(&spec)?,
        deploy_status: DeployStatus::Waiting.to_string(),
        deploy_message: "waiting to compile".to_string(),
        status: DeploymentStatus::Active.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };
    let mut active_model = model.into_active_model();
    active_model.id = Default::default();
    let db = DB.get().unwrap();
    let model = active_model.insert(db).await?;
    Ok(model)
}

/// is_deploying checks if a project is deploying
pub async fn is_deploying(project_id: i32) -> Result<bool> {
    let db = DB.get().unwrap();
    let dp = deployment::Entity::find()
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Status.eq(DeploymentStatus::Active.to_string()))
        .order_by_desc(deployment::Column::Id) // find latest one, need be success or failed
        .one(db)
        .await?;
    if dp.is_none() {
        return Ok(false);
    }
    let dp = dp.unwrap();
    Ok(dp.deploy_status == DeployStatus::Waiting.to_string()
        || dp.deploy_status == DeployStatus::Compiling.to_string()
        || dp.deploy_status == DeployStatus::Uploading.to_string()
        || dp.deploy_status == DeployStatus::Deploying.to_string())
}
