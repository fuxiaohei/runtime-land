use crate::{db::DB, models::settings, now_time};
use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};

static DOMAIN_SETTINGS: &str = "domain_settings";

#[derive(Serialize, Deserialize)]
pub struct DomainSettings {
    pub domain: String,
    pub protocol: String,
}

/// init_defaults initializes default settings
pub async fn init_defaults() -> Result<()> {
    let item = get(DOMAIN_SETTINGS).await?;
    if item.is_none() {
        let content = serde_json::to_string(&DomainSettings {
            domain: "127-0-0-1.nip.io".to_string(),
            protocol: "http".to_string(),
        })?;
        set(DOMAIN_SETTINGS, &content).await?;
    }
    Ok(())
}

/// get_domain_settings returns domain suffix and protocol
pub async fn get_domain_settings() -> Result<(String, String)> {
    let content = get(DOMAIN_SETTINGS).await?.unwrap().value;
    let settings: DomainSettings = serde_json::from_str(content.as_str())?;
    Ok((settings.domain, settings.protocol))
}

/// set_domain_settings sets domain suffix and protocol
pub async fn set_domain_settings(domain: String, protocol: String) -> Result<()> {
    let content = serde_json::to_string(&DomainSettings { domain, protocol })?;
    set(DOMAIN_SETTINGS, &content).await?;
    Ok(())
}

/// get settings item
pub async fn get(name: &str) -> Result<Option<settings::Model>> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(item)
}

/// set settings item
pub async fn set(name: &str, value: &str) -> Result<()> {
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
