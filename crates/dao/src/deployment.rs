use crate::db::DB;
use crate::models::deployment;
use crate::now_time;
use anyhow::Result;
use sea_orm::sea_query::Expr;
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
    Compiling, // if compilation is long time, we need mark it as compiling
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

/// list_by_status gets all deployments by status
pub async fn list_by_status(status: DeployStatus) -> Result<Vec<deployment::Model>> {
    let db = DB.get().unwrap();
    let dps = deployment::Entity::find()
        .filter(deployment::Column::DeployStatus.eq(status.to_string()))
        .filter(deployment::Column::Status.eq(DeploymentStatus::Active.to_string()))
        .order_by_desc(deployment::Column::Id)
        .all(db)
        .await?;
    Ok(dps)
}

/// set_failed sets a deployment as failed
pub async fn set_failed(id: i32, msg: String) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .filter(deployment::Column::Id.eq(id))
        .col_expr(
            deployment::Column::DeployStatus,
            Expr::value(DeployStatus::Failed.to_string()),
        )
        .col_expr(deployment::Column::DeployMessage, Expr::value(msg))
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

/// set_uploading sets a deployment as uploading
pub async fn set_uploading(id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .filter(deployment::Column::Id.eq(id))
        .col_expr(
            deployment::Column::DeployStatus,
            Expr::value(DeployStatus::Uploading.to_string()),
        )
        .col_expr(
            deployment::Column::DeployMessage,
            Expr::value(DeployStatus::Uploading.to_string()),
        )
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

/// set_uploaded sets a deployment as uploaded, waiting for deploying
pub async fn set_uploaded(id: i32, path: String, md5: String, size: i32) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .filter(deployment::Column::Id.eq(id))
        .col_expr(deployment::Column::StoragePath, Expr::value(path))
        .col_expr(deployment::Column::StorageMd5, Expr::value(md5))
        .col_expr(deployment::Column::StorageSize, Expr::value(size))
        .col_expr(
            deployment::Column::DeployStatus,
            Expr::value(DeployStatus::Deploying.to_string()),
        )
        .col_expr(
            deployment::Column::DeployMessage,
            Expr::value(DeployStatus::Deploying.to_string()),
        )
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}
