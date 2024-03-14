use crate::{models::worker, now_time, DB};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

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

/// update worker info. If not exist, create a new one.
pub async fn update(
    ip: &str,
    hostname: &str,
    ip_info: &str,
    machine_size: &str,
    status: Status,
) -> Result<worker::Model> {
    let now = now_time();
    let db = DB.get().unwrap();
    let info = worker::Entity::find()
        .filter(worker::Column::Ip.eq(ip))
        .one(db)
        .await?;
    if let Some(info) = info {
        let mut active_info: worker::ActiveModel = info.into();
        active_info.status = Set(status.to_string());
        active_info.updated_at = Set(now);
        let info = active_info.update(db).await?;
        return Ok(info);
    }
    let model = worker::Model {
        id: 0,
        ip: ip.to_string(),
        hostname: hostname.to_string(),
        ip_info: ip_info.to_string(),
        machine_size: machine_size.to_string(),
        status: status.to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut active_model: worker::ActiveModel = model.into();
    active_model.id = Default::default();
    let model = active_model.insert(db).await?;
    Ok(model)
}
