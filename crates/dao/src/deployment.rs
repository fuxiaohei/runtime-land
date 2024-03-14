use crate::{models::deployment, now_time, DB};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
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
    Uploading,
    Deploying,
    Success,
    Failed,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Deleted,
}

/// create a deployment
pub async fn create(user_id: i32, project_id: i32, domain: String) -> Result<deployment::Model> {
    let spec = Spec::default();
    let now = now_time();
    let model = deployment::Model {
        id: 0,
        user_id,
        project_id,
        domain,
        storage_path: "".to_string(),
        storage_md5: "".to_string(),
        storage_size: 0,
        spec: serde_json::to_value(&spec)?,
        deploy_status: DeployStatus::Uploading.to_string(),
        deploy_message: "".to_string(),
        status: Status::Active.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        task_id: uuid::Uuid::new_v4().to_string(),
    };
    let mut active_model = model.into_active_model();
    active_model.id = Default::default();
    let db = DB.get().unwrap();
    let model = active_model.insert(db).await?;
    Ok(model)
}

/// get_by_project gets a deployment by project
pub async fn get_by_project(user_id: i32, project_id: i32) -> Result<Option<deployment::Model>> {
    let model = deployment::Entity::find()
        .filter(deployment::Column::UserId.eq(user_id))
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Status.eq(Status::Active.to_string()))
        .one(DB.get().unwrap())
        .await?;
    Ok(model)
}

pub struct StorageInfo {
    pub path: String,
    pub md5: String,
    pub size: i32,
}

/// mark_status set deployment status
pub async fn mark_status(
    id: i32,
    status: DeployStatus,
    msg: String,
    task_id: Option<String>,
    storage_info: Option<StorageInfo>,
) -> Result<()> {
    let db = DB.get().unwrap();
    let mut updater = deployment::Entity::update_many()
        .filter(deployment::Column::Id.eq(id))
        .col_expr(
            deployment::Column::DeployStatus,
            Expr::value(status.to_string()),
        )
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .col_expr(deployment::Column::DeployMessage, Expr::value(msg));
    if let Some(task_id) = task_id {
        updater = updater.col_expr(deployment::Column::TaskId, Expr::value(task_id));
    }
    if let Some(info) = storage_info {
        updater = updater
            .col_expr(deployment::Column::StoragePath, Expr::value(info.path))
            .col_expr(deployment::Column::StorageMd5, Expr::value(info.md5))
            .col_expr(deployment::Column::StorageSize, Expr::value(info.size));
    }
    updater.exec(db).await?;
    Ok(())
}

/// list_active gets all active deployments
pub async fn list_active() -> Result<Vec<deployment::Model>> {
    // deploying and success status items
    let models = deployment::Entity::find()
        .filter(deployment::Column::Status.eq(Status::Active.to_string()))
        .filter(deployment::Column::DeployStatus.is_in(vec![
            DeployStatus::Deploying.to_string(),
            DeployStatus::Success.to_string(),
        ]))
        .order_by_desc(deployment::Column::Id)
        .all(DB.get().unwrap())
        .await?;
    Ok(models)
}

/// list_deploying gets all deploying deployments
pub async fn list_deploying() -> Result<Vec<deployment::Model>> {
    let models = deployment::Entity::find()
        .filter(deployment::Column::Status.eq(Status::Active.to_string()))
        .filter(deployment::Column::DeployStatus.eq(DeployStatus::Deploying.to_string()))
        .order_by_desc(deployment::Column::Id)
        .all(DB.get().unwrap())
        .await?;
    Ok(models)
}

/// get_latest_one gets the latest deployment
pub async fn get_latest_one() -> Result<Option<deployment::Model>> {
    let model = deployment::Entity::find()
        .order_by_desc(deployment::Column::UpdatedAt)
        .one(DB.get().unwrap())
        .await?;
    Ok(model)
}
