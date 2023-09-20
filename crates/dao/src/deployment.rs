use crate::model::project;
use crate::{model::deployment, DB};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, FromQueryResult, JsonValue, QueryFilter,
    QueryOrder, Set, Statement,
};
use std::collections::HashMap;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    InActive,
    Deleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeployStatus {
    Deploying,
    Success,
    Failed,
}

/// create creates a deployment
pub async fn create(
    owner_id: i32,
    project_id: i32,
    project_name: String,
    storage_path: String,
) -> Result<deployment::Model> {
    let project = crate::project::find_by_id(project_id)
        .await?
        .ok_or(anyhow::anyhow!("project not found"))?;
    let now = chrono::Utc::now();
    let uuid = uuid::Uuid::new_v4().to_string();
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let deployment_name = format!("{}-{}", project_name, rand_string.to_lowercase());
    let deployment = deployment::Model {
        id: 0,
        owner_id,
        project_id,
        project_uuid: project.uuid,
        domain: deployment_name,
        prod_domain: String::new(),
        uuid,
        storage_path,
        created_at: now,
        updated_at: now,
        status: Status::Active.to_string(),
        deploy_status: DeployStatus::Deploying.to_string(),
        deleted_at: None,
    };
    let active_model: deployment::ActiveModel = deployment.into();
    let db = DB.get().unwrap();
    let deployment = active_model.insert(db).await?;

    Ok(deployment)
}

pub async fn set_storage_success(id: i32, storage_path: String) -> Result<deployment::Model> {
    let db = DB.get().unwrap();
    let deployment = deployment::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(anyhow::anyhow!("deployment not found"))?;

    let mut active_model: deployment::ActiveModel = deployment.into();
    active_model.deploy_status = Set(DeployStatus::Success.to_string());
    active_model.storage_path = Set(storage_path);
    active_model.updated_at = Set(chrono::Utc::now());
    let deployment = active_model.update(db).await?;
    Ok(deployment)
}

/// set_deploy_status sets deploy status
pub async fn set_deploy_status(id: i32, status: DeployStatus) -> Result<()> {
    let db = DB.get().unwrap();
    let deployment = deployment::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(anyhow::anyhow!("deployment not found"))?;

    let mut active_model: deployment::ActiveModel = deployment.into();
    active_model.deploy_status = Set(status.to_string());
    active_model.updated_at = Set(chrono::Utc::now());
    active_model.update(db).await?;
    Ok(())
}

/// find_by_uuid finds a deployment by uuid
pub async fn publish(owner_id: i32, uuid: String) -> Result<deployment::Model> {
    let db = DB.get().unwrap();
    let deployment = deployment::Entity::find()
        .filter(deployment::Column::Uuid.eq(uuid))
        .filter(deployment::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?
        .ok_or(anyhow::anyhow!("deployment not found"))?;

    let project = crate::project::find_by_id(deployment.project_id)
        .await?
        .ok_or(anyhow::anyhow!("project not found"))?;

    // update old deployment domain
    deployment::Entity::update_many()
        .col_expr(deployment::Column::ProdDomain, Expr::value(String::new()))
        .filter(deployment::Column::ProjectId.eq(project.id))
        .exec(db)
        .await?;

    let mut active_model: deployment::ActiveModel = deployment.into();
    active_model.updated_at = Set(chrono::Utc::now());
    active_model.prod_domain = Set(project.name.clone());
    active_model.status = Set(Status::Active.to_string()); // force set status to active when publish
    let deployment = active_model.update(db).await?;

    // update project project.prod_deploy_id
    let mut project_active_model: project::ActiveModel = project.into();
    project_active_model.prod_deploy_id = Set(deployment.id);
    project_active_model.updated_at = Set(chrono::Utc::now());
    project_active_model.update(db).await?;

    Ok(deployment)
}

async fn set_status(owner_id: i32, uuid: String, status: Status) -> Result<deployment::Model> {
    let db = DB.get().unwrap();
    let deployment = deployment::Entity::find()
        .filter(deployment::Column::Uuid.eq(uuid))
        .filter(deployment::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?
        .ok_or(anyhow::anyhow!("deployment not found"))?;

    let mut active_model: deployment::ActiveModel = deployment.into();
    active_model.status = Set(status.to_string());
    active_model.updated_at = Set(chrono::Utc::now());
    let deployment = active_model.update(db).await?;
    Ok(deployment)
}

/// disable disables a deployment, set status to inactive
pub async fn disable(owner_id: i32, uuid: String) -> Result<deployment::Model> {
    set_status(owner_id, uuid, Status::InActive).await
}

/// enable enables a deployment, set status to active
pub async fn enable(owner_id: i32, uuid: String) -> Result<deployment::Model> {
    set_status(owner_id, uuid, Status::Active).await
}

/// list_counter lists the counter of deployments
pub async fn list_counter(owner_id: i32) -> Result<HashMap<i32, usize>> {
    let db = DB.get().unwrap();
    let values: Vec<JsonValue> = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"select count(id) as counter, project_id from deployment where owner_id = ? and status != 'deleted' group by project_id"#,
        [owner_id.into()],
    ))
    .all(db)
    .await?;
    let mut map = HashMap::new();
    for value in values {
        let counter = value["counter"].as_i64().unwrap() as usize;
        let project_id = value["project_id"].as_i64().unwrap() as i32;
        map.insert(project_id, counter);
    }
    Ok(map)
}

/// find_by_id finds a deployment by id
pub async fn find_by_id(owner_id: i32, id: i32) -> Result<Option<deployment::Model>> {
    let db = DB.get().unwrap();
    let deployment = deployment::Entity::find_by_id(id)
        .filter(deployment::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?;
    Ok(deployment)
}

/// list_active lists the success deployments, deploy status is success, status is active
pub async fn list_success() -> Result<Vec<deployment::Model>> {
    let db = DB.get().unwrap();
    let deployments = deployment::Entity::find()
        .filter(deployment::Column::Status.eq(Status::Active.to_string()))
        .filter(deployment::Column::DeployStatus.eq(DeployStatus::Success.to_string()))
        .all(db)
        .await?;
    Ok(deployments)
}

/// is_recent_updated checks if the deployment is recent updated
pub async fn is_recent_updated() -> Result<bool> {
    let db = DB.get().unwrap();
    let deployments = deployment::Entity::find()
        .filter(deployment::Column::DeployStatus.eq(DeployStatus::Success.to_string()))
        .order_by_desc(deployment::Column::UpdatedAt)
        .one(db)
        .await?;
    if deployments.is_none() {
        return Ok(false);
    }
    let deployment = deployments.unwrap();
    let now = chrono::Utc::now();
    let updated_at = deployment.updated_at;
    let duration = now.signed_duration_since(updated_at);
    let duration = duration.num_seconds();
    if duration > 60 {
        return Ok(false);
    }
    Ok(true)
}

/// list_by_project_id lists the deployments by project without deleted
pub async fn list_by_project_id(project_id: i32) -> Result<Vec<deployment::Model>> {
    let db = DB.get().unwrap();
    let deployments = deployment::Entity::find()
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(deployment::Column::CreatedAt)
        .all(db)
        .await?;
    Ok(deployments)
}

/// set_deleted_by_project sets the deployments deleted by project
pub async fn set_deleted_by_project(project_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .col_expr(
            deployment::Column::Status,
            Expr::value(Status::Deleted.to_string()),
        )
        .col_expr(
            deployment::Column::DeletedAt,
            Expr::value(chrono::Utc::now()),
        )
        .col_expr(
            deployment::Column::UpdatedAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Status.ne(Status::Deleted.to_string()))
        .exec(db)
        .await?;
    Ok(())
}

/// update_prod_domain updates the prod domain of deployment
pub async fn update_prod_domain(id: i32, domain: String) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .col_expr(deployment::Column::ProdDomain, Expr::value(domain))
        .col_expr(
            deployment::Column::UpdatedAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(deployment::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// get_stats gets the stats of deployments
pub async fn get_stats() -> Result<i32> {
    let db = DB.get().unwrap();
    let values: Vec<JsonValue> = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"select count(id) as counter from deployment where status != 'deleted'"#,
        [],
    ))
    .all(db)
    .await?;
    let counter = values[0]["counter"].as_i64().unwrap() as i32;
    Ok(counter)
}
