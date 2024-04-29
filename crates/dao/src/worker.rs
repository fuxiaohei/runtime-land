use crate::{db::DB, models::worker, now_time};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tracing::info;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Online,
    Offline,
}

/// list_online returns all online workers
pub async fn list_online() -> Result<Vec<worker::Model>> {
    let db = DB.get().unwrap();
    let workers = worker::Entity::find()
        .filter(worker::Column::Status.eq(Status::Online.to_string()))
        .all(db)
        .await?;
    Ok(workers)
}

/// list_all returns all workers
pub async fn list_all() -> Result<Vec<worker::Model>> {
    let db = DB.get().unwrap();
    let workers = worker::Entity::find()
        .order_by_desc(worker::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(workers)
}

/// update_online updates worker status
pub async fn update_online(
    ip: String,
    region: String,
    hostname: String,
    ip_info: String,
    machine_info: String,
    status: Status,
) -> Result<worker::Model> {
    let now = now_time();
    let db = DB.get().unwrap();
    let info = worker::Entity::find()
        .filter(worker::Column::Ip.eq(&ip))
        .one(db)
        .await?;
    if let Some(info) = info {
        if info.status != Status::Online.to_string() {
            info!("Set worker online: {}", ip);
        }
        let mut active_info: worker::ActiveModel = info.into();
        active_info.hostname = Set(hostname);
        active_info.status = Set(status.to_string());
        active_info.updated_at = Set(now);
        let info = active_info.update(db).await?;
        return Ok(info);
    }
    info!("Create worker: {}", ip);
    let model = worker::Model {
        id: 0,
        region,
        ip,
        ipv6: String::new(),
        hostname,
        ip_info,
        machine_info,
        status: status.to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut active_model: worker::ActiveModel = model.into();
    active_model.id = Default::default();
    let model = active_model.insert(db).await?;
    Ok(model)
}

/// set_offline set worker offline by id
pub async fn set_offline(id: i32) -> Result<()> {
    worker::Entity::update_many()
        .filter(worker::Column::Id.eq(id))
        .col_expr(
            worker::Column::Status,
            Expr::value(Status::Offline.to_string()),
        )
        .exec(DB.get().unwrap())
        .await?;
    Ok(())
}
