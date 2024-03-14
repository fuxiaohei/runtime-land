use crate::{models::deploy_task, now_time, DB};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};

#[derive(strum::Display, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Pending,
    Success,
    Failed,
}

/// create a deploy task
pub async fn create(
    deploy_id: i32,
    worker_id: i32,
    worker_ip: String,
    project_id: i32,
    task_id: String,
) -> Result<deploy_task::Model> {
    let now = now_time();
    let m = deploy_task::Model {
        id: 0,
        ip: worker_ip,
        worker_id,
        project_id,
        deployment_id: deploy_id,
        task_id,
        status: Status::Pending.to_string(),
        created_at: now,
        updated_at: now,
        message: String::new(),
    };
    let mut active_model = m.into_active_model();
    active_model.id = Default::default();
    let m2 = active_model.insert(DB.get().unwrap()).await?;
    Ok(m2)
}

/// update_pending updates the status of a deploy task
pub async fn update_pending(
    ip: String,
    task_id: String,
    status: Status,
    message: String,
) -> Result<()> {
    deploy_task::Entity::update_many()
        .filter(deploy_task::Column::Ip.eq(ip))
        .filter(deploy_task::Column::TaskId.eq(task_id))
        .filter(deploy_task::Column::Status.eq(Status::Pending.to_string())) // only change pending records
        .col_expr(deploy_task::Column::Status, Expr::value(status.to_string()))
        .col_expr(deploy_task::Column::UpdatedAt, Expr::value(now_time()))
        .col_expr(deploy_task::Column::Message, Expr::value(message))
        .exec(DB.get().unwrap())
        .await?;
    Ok(())
}

/// list_by_task_id returns all deploy tasks by task_id
pub async fn list_by_task_id(deploy_id: i32, task_id: String) -> Result<Vec<deploy_task::Model>> {
    let db = DB.get().unwrap();
    let tasks = deploy_task::Entity::find()
        .filter(deploy_task::Column::DeploymentId.eq(deploy_id))
        .filter(deploy_task::Column::TaskId.eq(task_id))
        .all(db)
        .await?;
    Ok(tasks)
}
