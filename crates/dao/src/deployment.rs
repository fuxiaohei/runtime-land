use crate::db::DB;
use crate::models::{deployment, deployment_task, project};
use crate::now_time;
use anyhow::Result;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

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
    Disabled, // if a deployment is disabled, it will not be deployed
    Deleted,  // if a deployment is deleted, it will not be shown
    Outdated, // if a deployment is outdated, it will be deleted
}

/// get_last_by_project gets the last deployment by project
pub async fn get_last_by_project(project_id: i32) -> Result<Option<deployment::Model>> {
    let db = DB.get().unwrap();
    let dp = deployment::Entity::find()
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Status.eq(DeploymentStatus::Active.to_string()))
        .order_by_desc(deployment::Column::Id) // find latest one, need be success or failed
        .one(db)
        .await?;
    Ok(dp)
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
pub async fn set_failed(id: i32, project_id: i32, msg: String) -> Result<()> {
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
    project::Entity::update_many()
        .filter(project::Column::Id.eq(project_id))
        .col_expr(
            project::Column::DeployStatus,
            Expr::value(DeployStatus::Failed.to_string()),
        )
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

async fn set_status(id: i32, project_id: i32, status: DeployStatus) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .filter(deployment::Column::Id.eq(id))
        .col_expr(
            deployment::Column::DeployStatus,
            Expr::value(status.to_string()),
        )
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    project::Entity::update_many()
        .filter(project::Column::Id.eq(project_id))
        .col_expr(
            project::Column::DeployStatus,
            Expr::value(status.to_string()),
        )
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

/// set_uploading sets a deployment as uploading
pub async fn set_uploading(id: i32, project_id: i32) -> Result<()> {
    set_status(id, project_id, DeployStatus::Uploading).await
}

/// set_compiling sets a deployment as compiling
pub async fn set_compiling(id: i32, project_id: i32) -> Result<()> {
    set_status(id, project_id, DeployStatus::Compiling).await
}

/// set_success sets a deployment as success
pub async fn set_success(id: i32, project_id: i32) -> Result<()> {
    set_status(id, project_id, DeployStatus::Success).await?;
    // set old deployments as outdated
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Id.ne(id))
        .col_expr(
            deployment::Column::Status,
            Expr::value(DeploymentStatus::Outdated.to_string()),
        )
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

/// set_uploaded sets a deployment as uploaded, waiting for deploying
pub async fn set_uploaded(
    id: i32,
    project_id: i32,
    path: String,
    md5: String,
    size: i32,
) -> Result<()> {
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
    project::Entity::update_many()
        .filter(project::Column::Id.eq(project_id))
        .col_expr(
            project::Column::DeployStatus,
            Expr::value(DeployStatus::Deploying.to_string()),
        )
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

/// list_tasks_by_taskid gets all tasks by task_id
pub async fn list_tasks_by_taskid(task_id: String) -> Result<Vec<deployment_task::Model>> {
    let db = DB.get().unwrap();
    let tasks = deployment_task::Entity::find()
        .filter(deployment_task::Column::TaskId.eq(task_id))
        .all(db)
        .await?;
    Ok(tasks)
}

/// create_task creates a task
pub async fn create_task(
    worker_id: i32,
    ip: String,
    project_id: i32,
    deployment_id: i32,
    task_id: String,
    content: String,
) -> Result<deployment_task::Model> {
    let now = now_time();
    let model = deployment_task::Model {
        id: 0,
        worker_id,
        ip,
        project_id,
        deployment_id,
        task_id,
        content,
        deploy_status: DeployStatus::Deploying.to_string(),
        deploy_message: "deploying".to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut active_model = model.into_active_model();
    active_model.id = Default::default();
    let db = DB.get().unwrap();
    let model = active_model.insert(db).await?;
    Ok(model)
}

/// list_tasks_by_ip gets all tasks by ip
pub async fn list_tasks_by_ip(
    ip: String,
    status: Option<DeployStatus>,
) -> Result<Vec<deployment_task::Model>> {
    let db = DB.get().unwrap();
    let mut select = deployment_task::Entity::find().filter(deployment_task::Column::Ip.eq(ip));
    if let Some(s) = status {
        select = select.filter(deployment_task::Column::DeployStatus.eq(s.to_string()));
    }
    let tasks = select
        .order_by_desc(deployment_task::Column::Id)
        .all(db)
        .await?;
    Ok(tasks)
}

/// update_task_result updates task result
pub async fn update_task_result(task_id: String, ip: String, result: String) -> Result<()> {
    let task = get_task(task_id.clone(), ip.clone()).await?;
    if task.is_none() {
        debug!("Task not found, task_id: {}, ip: {}", task_id, ip);
        return Ok(());
    }
    let task = task.unwrap();
    if task.deploy_status == DeployStatus::Success.to_string() {
        debug!("Task already success, task_id: {}, ip: {}", task_id, ip);
        return Ok(());
    }

    // if result is success, update as success
    if result == "success" {
        deployment_task::Entity::update_many()
            .filter(deployment_task::Column::TaskId.eq(task_id.clone()))
            .filter(deployment_task::Column::Ip.eq(ip.clone()))
            .col_expr(
                deployment_task::Column::DeployStatus,
                Expr::value(DeployStatus::Success.to_string()),
            )
            .col_expr(
                deployment_task::Column::DeployMessage,
                Expr::value("success"),
            )
            .col_expr(deployment_task::Column::UpdatedAt, Expr::value(now_time()))
            .exec(DB.get().unwrap())
            .await?;
        info!("Task success, task_id: {}, ip: {}", task_id, ip);
    } else {
        deployment_task::Entity::update_many()
            .filter(deployment_task::Column::TaskId.eq(task_id.clone()))
            .filter(deployment_task::Column::Ip.eq(ip.clone()))
            .col_expr(
                deployment_task::Column::DeployStatus,
                Expr::value(DeployStatus::Failed.to_string()),
            )
            .col_expr(
                deployment_task::Column::DeployMessage,
                Expr::value(result.clone()),
            )
            .col_expr(deployment_task::Column::UpdatedAt, Expr::value(now_time()))
            .exec(DB.get().unwrap())
            .await?;
        info!(
            "Task failed, task_id: {}, ip: {}, msg: {}",
            task_id, ip, result
        );
    }
    Ok(())
}

async fn get_task(task_id: String, ip: String) -> Result<Option<deployment_task::Model>> {
    let db = DB.get().unwrap();
    let task = deployment_task::Entity::find()
        .filter(deployment_task::Column::TaskId.eq(task_id))
        .filter(deployment_task::Column::Ip.eq(ip))
        .one(db)
        .await?;
    Ok(task)
}
