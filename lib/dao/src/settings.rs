use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::{models::settings, now_time, DB};

/// get settings item and deserialize it as json
pub async fn get<T>(name: &str) -> Result<Option<T>>
where
    for<'a> T: Deserialize<'a>,
{
    let item = get_value(name).await?;
    match item {
        Some(item) => {
            let value = serde_json::from_str(&item.value)?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// get settings item
pub async fn get_value(name: &str) -> Result<Option<settings::Model>> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(item)
}

/// set sets name, and serialized value  into settings table
pub async fn set(name: &str, value: impl Serialize) -> Result<()> {
    let value = serde_json::to_string(&value)?;
    set_value(name, &value).await
}

/// set settings item
pub async fn set_value(name: &str, value: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let now = now_time();
    if item.is_none() {
        let item = settings::ActiveModel {
            name: Set(name.to_string()),
            value: Set(value.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        item.insert(db).await?;
    } else {
        let item = item.unwrap();
        let mut item = item.into_active_model();
        item.value = Set(value.to_string());
        item.updated_at = Set(now);
        item.save(db).await?;
    }
    Ok(())
}
