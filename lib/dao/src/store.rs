use std::collections::HashMap;

use crate::{models::storage, now_time, DB};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Uploading,
    Success,
    LocalDeleted,
    RemoteDeleted,
}

/// create create storage
pub async fn create(
    owner_id: i32,
    project_id: i32,
    deploy_id: i32,
    task_id: &str,
    file_path: &str,
    file_hash: &str,
    file_size: i32,
) -> Result<storage::Model> {
    let now = now_time();
    let model = storage::Model {
        id: 0,
        owner_id,
        project_id,
        deploy_id,
        task_id: task_id.to_string(),
        path: file_path.to_string(),
        file_hash: file_hash.to_string(),
        file_size,
        file_target: String::new(),
        status: Status::Uploading.to_string(),
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

/// set_success set storage status to normal
pub async fn set_success(id: i32, target: Option<String>) -> Result<()> {
    let db = DB.get().unwrap();
    storage::Entity::update_many()
        .col_expr(
            storage::Column::Status,
            Expr::value(Status::Success.to_string()),
        )
        .col_expr(
            storage::Column::FileTarget,
            Expr::value(target.unwrap_or_default()),
        )
        .filter(storage::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// list_success_by_deploys list success storage by deploy ids
pub async fn list_success_by_deploys(deploy_ids: Vec<i32>) -> Result<HashMap<i32, storage::Model>> {
    let db = DB.get().unwrap();
    let models = storage::Entity::find()
        .filter(storage::Column::DeployId.is_in(deploy_ids))
        .filter(storage::Column::Status.eq(Status::Success.to_string()))
        .order_by_asc(storage::Column::Id)
        .all(db)
        .await?;
    let mut map = HashMap::new();
    for model in models {
        map.insert(model.deploy_id, model);
    }
    Ok(map)
}
