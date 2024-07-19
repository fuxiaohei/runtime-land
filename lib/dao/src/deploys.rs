use crate::{models::deployment, now_time, DB};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::{Deserialize, Serialize};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
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

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeployType {
    Production,  // production deployment
    Development, // development deployment
}

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

/// create a deployment
pub async fn create(
    owner_id: i32,
    owner_uuid: String,
    project_id: i32,
    project_uuid: String,
    domain: String,
    deploy_type: DeployType,
) -> Result<deployment::Model> {
    let spec = Spec::default();
    let now = now_time();
    let model = deployment::Model {
        id: 0,
        owner_id,
        owner_uuid,
        project_id,
        project_uuid,
        task_id: uuid::Uuid::new_v4().to_string(),
        domain,
        spec: serde_json::to_value(&spec)?,
        deploy_type: deploy_type.to_string(),
        deploy_status: Status::Waiting.to_string(),
        deploy_message: "Waiting to deploy".to_string(),
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

/// list_by_deploy_status returns a list of deployments by deploy status
pub async fn list_by_deploy_status(status: Status) -> Result<Vec<deployment::Model>> {
    let db = DB.get().unwrap();
    let models = deployment::Entity::find()
        .filter(deployment::Column::DeployStatus.contains(status.to_string()))
        .all(db)
        .await?;
    Ok(models)
}

/// set_deploy_status sets the status of a deployment
pub async fn set_deploy_status(deploy_id: i32, status: Status, message: &str) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .col_expr(
            deployment::Column::DeployStatus,
            Expr::value(status.to_string()),
        )
        .col_expr(deployment::Column::DeployMessage, Expr::value(message))
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .filter(deployment::Column::Id.eq(deploy_id))
        .exec(db)
        .await?;
    Ok(())
}
