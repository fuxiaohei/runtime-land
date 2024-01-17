use crate::models::settings;
use crate::DB;
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

/// init_settings initializes default settings
pub async fn init_settings() -> Result<()> {
    let item = get("domain_suffix").await?;
    if item.is_none() {
        set("domain_suffix", "runtime.lol").await?;
    }
    let item = get("domain_protocol").await?;
    if item.is_none() {
        set("domain_protocol", "http").await?;
    }
    Ok(())
}

/// get_domain_settings returns domain suffix and protocol
pub async fn get_domain_settings() -> Result<(String, String)> {
    let suffix = get("domain_suffix").await?.unwrap().value;
    let protocol = get("domain_protocol").await?.unwrap().value;
    Ok((suffix, protocol))
}

/// set_domain_settings sets domain suffix and protocol
pub async fn set_domain_settings(suffix: String, protocol: String) -> Result<()> {
    set("domain_suffix", suffix.as_str()).await?;
    set("domain_protocol", protocol.as_str()).await?;
    Ok(())
}

/// set_confs_refresh_flag sets confs_refresh_flag
pub async fn set_confs_refresh_flag() -> Result<()> {
    let now_ts = chrono::Utc::now().timestamp().to_string();
    set("confs_refresh_flag", &now_ts).await?;
    Ok(())
}

/// get_confs_refresh_flag returns confs_refresh_flag
pub async fn get_confs_refresh_flag() -> Result<i64> {
    let item = get("confs_refresh_flag").await?;
    if item.is_none() {
        return Ok(0);
    }
    let item = item.unwrap();
    let flag = item.value.parse::<i64>()?;
    Ok(flag)
}

pub async fn get(name: &str) -> Result<Option<settings::Model>> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(item)
}

pub async fn set(name: &str, value: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let item = settings::Entity::find()
        .filter(settings::Column::Name.eq(name))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let now = chrono::Utc::now();
    if item.is_none() {
        let item = settings::ActiveModel {
            id: Set(0),
            name: Set(name.to_string()),
            label: Set(name.to_string()),
            value: Set(value.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
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
