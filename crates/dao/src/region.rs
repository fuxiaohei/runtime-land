use crate::{model::region, DB};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use std::collections::HashMap;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    InActive,
    Empty,
    Deleted,
}

/// list_maps lists all regions as hashmap with key field and by owner id
pub async fn list_maps() -> Result<HashMap<String, region::Model>> {
    let db = DB.get().unwrap();
    let regions = region::Entity::find()
        .filter(region::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_asc(region::Column::Key)
        .all(db)
        .await?;
    let mut regions_map = HashMap::new();
    for region in regions {
        regions_map.insert(region.key.clone(), region);
    }
    Ok(regions_map)
}

/// create_region creates a region
pub async fn create(region: super::Region) -> Result<()> {
    let active_model: region::ActiveModel = region.into();
    let db = DB.get().unwrap();
    active_model.insert(db).await?;
    Ok(())
}

async fn find_by_key(key: String) -> Result<region::Model> {
    let db = DB.get().unwrap();
    let region = region::Entity::find()
        .filter(region::Column::Key.eq(key))
        .one(db)
        .await?
        .ok_or(anyhow::anyhow!("region not found"))?;
    Ok(region)
}

/// update runtimes updates runtimes of region
pub async fn update_runtimes(key: String, runtimes: i32) -> Result<()> {
    let region = find_by_key(key).await?;
    let mut active_model: region::ActiveModel = region.into();
    active_model.runtimes = Set(runtimes);
    active_model.updated_at = Set(chrono::Utc::now());
    active_model.status = Set(Status::Active.to_string());
    if runtimes == 0 {
        active_model.status = Set(Status::Empty.to_string());
    }

    let db = DB.get().unwrap();
    active_model.update(db).await?;
    Ok(())
}

/// set_inactive sets region to inactive
pub async fn set_inactive(key: String) -> Result<()> {
    let region = find_by_key(key).await?;
    let mut active_model: region::ActiveModel = region.into();
    active_model.status = Set(Status::InActive.to_string());

    let db = DB.get().unwrap();
    active_model.update(db).await?;
    Ok(())
}
