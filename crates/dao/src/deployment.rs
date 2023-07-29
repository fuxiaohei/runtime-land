use crate::{model::deployment, DB};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::ActiveModelTrait;

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
        .take(10)
        .map(char::from)
        .collect();
    let deployment_name = format!("{}-{}", project_name, rand_string);
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
