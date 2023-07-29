use crate::{model::settings, DB};
use anyhow::Result;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;

#[derive(strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Key {
    ProductionDomain,
    ProductionProtocol,
}

/// list_maps lists settings with key field as hashmap
pub async fn list_maps(keys: Vec<String>) -> Result<HashMap<String, String>> {
    let db = DB.get().unwrap();
    let settings = settings::Entity::find()
        .filter(settings::Column::Key.is_in(keys))
        .all(db)
        .await?;
    let mut settings_map = HashMap::new();
    for setting in settings {
        settings_map.insert(setting.key.clone(), setting.value);
    }
    Ok(settings_map)
}

/// update updates settings
pub async fn update(values: Vec<settings::Model>) -> Result<()> {
    let db = DB.get().unwrap();

    // delete old keys
    let keys = values
        .iter()
        .map(|setting| setting.key.clone())
        .collect::<Vec<String>>();
    settings::Entity::delete_many()
        .filter(settings::Column::Key.is_in(keys))
        .exec(db)
        .await?;

    // insert new keys
    let active_models: Vec<settings::ActiveModel> =
        values.into_iter().map(|setting| setting.into()).collect();
    crate::model::prelude::Settings::insert_many(active_models)
        .exec(db)
        .await?;

    Ok(())
}
