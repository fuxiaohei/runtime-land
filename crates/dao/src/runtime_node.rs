use crate::{model::runtime_node, DB};
use anyhow::Result;
use land_core::confdata::RuntimeNodeInfo;
use sea_orm::prelude::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Online,
    Offline,
    Deleted,
}

// into converts runtime node model to runtime node info
impl From<runtime_node::Model> for RuntimeNodeInfo {
    fn from(val: runtime_node::Model) -> Self {
        RuntimeNodeInfo {
            ip: val.ip,
            city: val.city,
            country: val.country,
            region: val.region,
            loc: String::new(),
            org: String::new(),
            timezone: String::new(),
            readme: String::new(),
            conf_hash: Some(val.conf_hash),
        }
    }
}

// into converts runtime node info to runtime node model
impl From<RuntimeNodeInfo> for runtime_node::Model {
    fn from(val: RuntimeNodeInfo) -> Self {
        let now = chrono::Utc::now();
        runtime_node::Model {
            id: 0,
            name: val.region(),
            key: val.region_ip(),
            ip: val.ip,
            city: val.city,
            region: val.region,
            country: val.country,
            status: Status::Online.to_string(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            conf_hash: String::new(),
        }
    }
}

/// get_all_map gets all runtime node info as hashmap
pub async fn get_all_map() -> Result<HashMap<String, RuntimeNodeInfo>> {
    let db = DB.get().unwrap();
    let nodes = runtime_node::Entity::find()
        .order_by_desc(runtime_node::Column::UpdatedAt)
        .all(db)
        .await?;
    let mut result = HashMap::new();
    for node in nodes {
        let info: RuntimeNodeInfo = node.into();
        result.insert(info.region_ip(), info);
    }
    Ok(result)
}

/// create creates a runtime node
pub async fn create(info: RuntimeNodeInfo, conf_md5: String) -> Result<()> {
    let mut model: runtime_node::Model = info.into();
    model.conf_hash = conf_md5;
    let active_model: runtime_node::ActiveModel = model.into();
    let db = DB.get().unwrap();
    active_model.insert(db).await?;
    Ok(())
}

/// update_onlines updates one online runtime node
pub async fn update_online(key: String, conf_md5: String) -> Result<()> {
    let db = DB.get().unwrap();
    runtime_node::Entity::update_many()
        .filter(runtime_node::Column::Key.eq(key))
        .col_expr(runtime_node::Column::ConfHash, Expr::value(conf_md5))
        .col_expr(
            runtime_node::Column::UpdatedAt,
            Expr::value(chrono::Utc::now()),
        )
        .col_expr(
            runtime_node::Column::Status,
            Expr::value(Status::Online.to_string()),
        )
        .exec(db)
        .await?;
    Ok(())
}

/// update_offline updates offline runtime nodes if expired
pub async fn update_offline(expired: u64) -> Result<()> {
    let db = DB.get().unwrap();
    let expired_time = chrono::Utc::now() - chrono::Duration::seconds(expired as i64);
    runtime_node::Entity::update_many()
        .filter(runtime_node::Column::UpdatedAt.lt(expired_time))
        .filter(runtime_node::Column::Status.eq(Status::Online.to_string())) // only for online node
        .col_expr(
            runtime_node::Column::Status,
            Expr::value(Status::Offline.to_string()),
        )
        .exec(db)
        .await?;
    Ok(())
}
