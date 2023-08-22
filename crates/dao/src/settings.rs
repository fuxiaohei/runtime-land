use crate::{model::settings, DB};
use anyhow::Result;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;

#[derive(strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Key {
    ProductionDomain,
    ProductionProtocol,
    StorageType,
    S3Storage,
    LocalStorage,
    EmailSmtp,
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

/// update_maps updates settings with key field as hashmap
pub async fn update_maps(map: HashMap<String, String>) -> Result<()> {
    let now = chrono::Utc::now();
    let values: Vec<settings::Model> = map
        .into_iter()
        .map(|(key, value)| settings::Model {
            id: 0,
            name: key.clone(),
            key,
            value,
            created_at: now,
            updated_at: now,
        })
        .collect();
    update(values).await
}

async fn update(values: Vec<settings::Model>) -> Result<()> {
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

/// EmailStmp is email stmp settings
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct EmailStmp {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub from: String,
}

impl Default for EmailStmp {
    fn default() -> Self {
        Self {
            host: "smtp.example.com".to_string(),
            port: "465".to_string(),
            username: "username".to_string(),
            password: "password".to_string(),
            from: "no-reploy@example.com".to_string(),
        }
    }
}

/// get_email_setting gets email stmp settings
pub async fn get_email_setting() -> EmailStmp {
    let db = DB.get().unwrap();
    let settings = settings::Entity::find()
        .filter(settings::Column::Key.eq(Key::EmailSmtp.to_string()))
        .one(db)
        .await
        .unwrap();
    if settings.is_none() {
        return EmailStmp::default();
    }
    let settings = settings.unwrap();
    let email_setting: EmailStmp = serde_json::from_str(&settings.value).unwrap();
    email_setting
}

/// update_email_setting updates email stmp settings
pub async fn update_email_setting(email: &EmailStmp) -> Result<()> {
    let key = Key::EmailSmtp.to_string();
    let value = serde_json::to_string(email)?;
    let values: HashMap<String, String> = vec![(key.clone(), value)].into_iter().collect();
    update_maps(values).await
}
