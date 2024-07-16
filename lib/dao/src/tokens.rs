use crate::{models::user_token, DB};
use anyhow::{anyhow, Result};
use land_common::rand_string;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::ops::Add;

/// TokenUsage is the usage of the token
#[derive(strum::Display, PartialEq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TokenUsage {
    Session, // web page session token
    Cmdline, // land-cli token
    Worker,  // land-worker token
}

/// TokenStatus is the status of the token
#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TokenStatus {
    Active,
    Deleted,
}

/// get_by_value gets an active token by value
pub async fn get_by_value(
    value: &str,
    usage: Option<TokenUsage>,
) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(value))
        .filter(user_token::Column::Status.eq(TokenStatus::Active.to_string()));
    if let Some(u) = usage {
        select = select.filter(user_token::Column::Usage.eq(u.to_string()));
    }
    let token = select.one(db).await.map_err(|e| anyhow!(e))?;
    Ok(token)
}

/// create creates a new token
pub async fn create(
    owner_id: i32,
    name: &str,
    expire: i64,
    usage: TokenUsage,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now();
    let expired_at = now.add(chrono::TimeDelta::try_seconds(expire).unwrap());
    let value: String = rand_string(40);
    let token_model = user_token::Model {
        id: 0,
        owner_id,
        value,
        name: name.to_string(),
        status: TokenStatus::Active.to_string(),
        created_at: now.naive_utc(),
        latest_used_at: now.naive_utc(),
        expired_at: Some(expired_at.naive_utc()),
        deleted_at: None,
        usage: usage.to_string(),
    };
    let mut token_active_model: user_token::ActiveModel = token_model.into();
    token_active_model.id = Default::default();
    let token_model = token_active_model.insert(DB.get().unwrap()).await?;
    Ok(token_model)
}
