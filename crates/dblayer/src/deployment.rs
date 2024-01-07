use crate::{models::project_deployment, DB};
use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

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
    Replaced,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeployStatus {
    Deploying,
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
        storage_path: String::new(),
        storage_md5: storage_md5.to_string(),
        trace_uuid,
        prod_status: DeploymentType::Testing.to_string(),
        deploy_status: DeployStatus::Deploying.to_string(),
        status: Status::Active.to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        specification: String::new(),
    };
    let project_active_model: project_deployment::ActiveModel = project.into();
    let project_model = project_active_model.insert(db).await?;
    Ok(project_model)
}
