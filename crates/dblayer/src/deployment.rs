use crate::{models::project_deployment, DB};
use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::QueryOrder;
use sea_orm::{sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeploymentType {
    Testing,
    Production,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Pending,
    Replaced,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeployStatus {
    Deploying,
    Success,
    Failed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Specification {
    cpu_time_limit: Option<i32>,
    memory_limit: Option<i32>,
    wall_time_limit: Option<i32>,
    fetch_limit: Option<i32>,
}

impl Default for Specification {
    fn default() -> Self {
        Self {
            cpu_time_limit: Some(100), // 100ms
            memory_limit: Some(128),   // 128MB
            wall_time_limit: Some(30), // 30 seconds
            fetch_limit: Some(5),      // send 5 requests
        }
    }
}

/// find_by_id returns the deployment by id
pub async fn find_by_id(id: i32) -> Result<Option<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let project = project_deployment::Entity::find()
        .filter(project_deployment::Column::Id.eq(id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(project)
}

pub async fn find_by_project(
    project_id: i32,
    status: DeploymentType,
) -> Result<Option<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let project = project_deployment::Entity::find()
        .filter(project_deployment::Column::ProjectId.eq(project_id))
        .filter(project_deployment::Column::ProdStatus.eq(status.to_string()))
        .filter(project_deployment::Column::Status.eq(Status::Active.to_string()))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(project)
}

pub async fn set_replaced(deploy_id: i32) -> Result<()> {
    set_status_internal(deploy_id, Status::Replaced).await
}

async fn set_status_internal(deploy_id: i32, to_status: Status) -> Result<()> {
    let db = DB.get().unwrap();
    project_deployment::Entity::update_many()
        .filter(project_deployment::Column::Id.eq(deploy_id))
        .col_expr(
            project_deployment::Column::Status,
            Expr::value(to_status.to_string()),
        )
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

pub async fn create(
    project_id: i32,
    owner_id: i32,
    project_name: &str,
    trace_uuid: &str,
    storage_md5: &str,
    storage_size: i32,
    storage_content_type: &str,
) -> Result<project_deployment::Model> {
    let random_string = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();
    let trace_uuid = if trace_uuid.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        trace_uuid.to_string()
    };
    let db = DB.get().unwrap();
    let project = project_deployment::Model {
        id: 0,
        project_id,
        owner_id,
        name: format!("{}-{}", project_name, random_string),
        project_name: project_name.to_string(),
        storage_path: String::new(),
        storage_size,
        storage_content_type: storage_content_type.to_string(),
        storage_md5: storage_md5.to_string(),
        trace_uuid,
        prod_status: DeploymentType::Testing.to_string(),
        deploy_status: DeployStatus::Deploying.to_string(),
        status: Status::Pending.to_string(), // default pending
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        specification: serde_json::to_value(Specification::default()).unwrap(),
    };
    let project_active_model: project_deployment::ActiveModel = project.into();
    let project_model = project_active_model.insert(db).await?;
    Ok(project_model)
}

pub async fn make_deploy_failed(deploy_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    project_deployment::Entity::update_many()
        .filter(project_deployment::Column::Id.eq(deploy_id))
        .col_expr(
            project_deployment::Column::DeployStatus,
            Expr::value(DeployStatus::Failed.to_string()),
        )
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

pub async fn make_deploy_success(
    deploy_id: i32,
    old_deploy_id: i32,
    storage_path: &str,
) -> Result<()> {
    let db = DB.get().unwrap();
    project_deployment::Entity::update_many()
        .filter(project_deployment::Column::Id.eq(deploy_id))
        .col_expr(
            project_deployment::Column::DeployStatus,
            Expr::value(DeployStatus::Success.to_string()),
        )
        .col_expr(
            project_deployment::Column::StoragePath,
            Expr::value(storage_path),
        )
        .col_expr(
            project_deployment::Column::Status,
            Expr::value(Status::Active.to_string()),
        )
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    if old_deploy_id > 0 {
        set_replaced(old_deploy_id).await?;
    }
    Ok(())
}

/// get_latest_updated returns all deployments updated in last `duration` seconds
pub async fn get_latest_updated(duration: i64) -> Result<Vec<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now();
    let now = now - chrono::Duration::seconds(duration);
    let projects = project_deployment::Entity::find()
        .filter(project_deployment::Column::UpdatedAt.gt(now))
        .filter(project_deployment::Column::Status.eq(Status::Active.to_string()))
        .order_by_desc(project_deployment::Column::Id)
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(projects)
}

/// list_actives returns all active deployments
pub async fn list_actives() -> Result<Vec<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let projects = project_deployment::Entity::find()
        .filter(project_deployment::Column::Status.eq(Status::Active.to_string()))
        .order_by_desc(project_deployment::Column::Id)
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(projects)
}
