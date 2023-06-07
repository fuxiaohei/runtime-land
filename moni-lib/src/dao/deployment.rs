use crate::db::DB;
use crate::model::{project_deployment, project_info};
use anyhow::Result;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};

use super::project;

enum DeploymentStatus {
    Deploying = 1,
    Deployed,
}

enum DeploymentProdStatus {
    Prod = 1,
    Preview,
}

pub async fn create(
    owner_id: i32,
    project_id: i32,
    name: String,
    storage_path: String,
) -> Result<project_deployment::Model> {
    let now = chrono::Utc::now();
    let uuid = uuid::Uuid::new_v4().to_string();
    let deployment = project_deployment::Model {
        id: 0,
        owner_id,
        project_id,
        domain: name,
        prod_domain: String::new(),
        uuid,
        storage_path,
        created_at: now,
        updated_at: now,
        prod_status: DeploymentProdStatus::Preview as i32,
        deploy_status: DeploymentStatus::Deploying as i32,
    };
    let active_model: project_deployment::ActiveModel = deployment.into();
    let db = DB.get().unwrap();
    let deployment = active_model.insert(db).await?;
    Ok(deployment)
}

pub async fn find(
    deploy_id: i32,
    deploy_uuid: String,
) -> Result<Option<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let deployment = project_deployment::Entity::find()
        .filter(project_deployment::Column::Id.eq(deploy_id))
        .filter(project_deployment::Column::Uuid.eq(deploy_uuid))
        .one(db)
        .await?;
    Ok(deployment)
}

pub async fn find_by_id(deploy_id: i32) -> Result<Option<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let deployment = project_deployment::Entity::find()
        .filter(project_deployment::Column::Id.eq(deploy_id))
        .one(db)
        .await?;
    Ok(deployment)
}

pub async fn update_storage(deploy_id: i32, storage_path: String) -> Result<()> {
    let db = DB.get().unwrap();
    let deployment = project_deployment::Entity::find()
        .filter(project_deployment::Column::Id.eq(deploy_id))
        .one(db)
        .await?;

    if deployment.is_none() {
        return Err(anyhow::anyhow!("deployment not found"));
    }

    let mut deployment_model: project_deployment::ActiveModel = deployment.unwrap().into();
    deployment_model.storage_path = Set(storage_path);
    deployment_model.deploy_status = Set(DeploymentStatus::Deployed as i32);
    deployment_model.update(db).await?;

    Ok(())
}

pub async fn promote(
    owner_id: i32,
    deploy_id: i32,
    deploy_uuid: String,
) -> Result<project_deployment::Model> {
    let deployment = find(deploy_id, deploy_uuid).await?;
    if deployment.is_none() {
        return Err(anyhow::anyhow!("deployment not found"));
    }

    // get project
    let deployment = deployment.unwrap();
    let project = project::find_by_id(owner_id, deployment.project_id).await?;
    if project.is_none() {
        return Err(anyhow::anyhow!("project not found"));
    }
    let project = project.unwrap();
    let prod_domain = project.name.clone();

    let db = DB.get().unwrap();
    let txn = db.begin().await?;

    // update project prod deployment id
    let mut project_model: project_info::ActiveModel = project.into();
    project_model.prod_deploy_id = Set(Some(deployment.id as i32));
    project_model.update(&txn).await?;

    // update all other deployments to preview
    project_deployment::Entity::update_many()
        .col_expr(
            project_deployment::Column::ProdStatus,
            Expr::value(DeploymentProdStatus::Preview as i32),
        )
        .filter(project_deployment::Column::Id.ne(deployment.id))
        .exec(&txn)
        .await?;

    // update current deployment to prod
    let mut deployment_model: project_deployment::ActiveModel = deployment.into();
    deployment_model.prod_status = Set(DeploymentProdStatus::Prod as i32);
    deployment_model.prod_domain = Set(prod_domain);
    let deployment = deployment_model.update(&txn).await?;

    txn.commit().await?;

    Ok(deployment)
}

pub async fn list(
    owner_id: i32,
    project_id: i32,
    limits: u64,
) -> Result<Vec<project_deployment::Model>> {
    let db = DB.get().unwrap();
    let deployments = project_deployment::Entity::find()
        .filter(project_deployment::Column::OwnerId.eq(owner_id))
        .filter(project_deployment::Column::ProjectId.eq(project_id))
        .order_by_desc(project_deployment::Column::UpdatedAt)
        .limit(limits)
        .all(db)
        .await?;
    Ok(deployments)
}
