use crate::{model::deployment, DB};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

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
