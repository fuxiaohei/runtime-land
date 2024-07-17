use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::info;

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

#[derive(Clone, Serialize, Deserialize)]
pub struct DomainSettings {
    pub domain_suffix: String,
    pub http_protocol: String,
}

static DOMAIN_SETTINGS_KEY: &str = "domain-settings";

/// get_domain_settings get domain settings
pub async fn get_domain_settings() -> Result<DomainSettings> {
    if let Some(settings) = get(DOMAIN_SETTINGS_KEY).await? {
        return Ok(settings);
    }
    Err(anyhow!("domain settings not found"))
}

/// set_domain_settings set domain settings
pub async fn set_domain_settings(domain_suffix: &str, http_protocol: &str) -> Result<()> {
    let settings = DomainSettings {
        domain_suffix: domain_suffix.to_string(),
        http_protocol: http_protocol.to_string(),
    };
    set(DOMAIN_SETTINGS_KEY, settings).await
}

/// init_defaults init defaults
pub async fn init_defaults() -> Result<()> {
    let v = get_value(DOMAIN_SETTINGS_KEY).await?;
    if v.is_none() {
        set_domain_settings("localhost", "http").await?;
        info!("init domain settings")
    }
    Ok(())
}
