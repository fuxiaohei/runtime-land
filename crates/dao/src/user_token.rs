use anyhow::Result;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use std::{collections::HashMap, ops::Add};
use tokio::sync::Mutex;

use crate::{models::user_token, DB};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Deleted,
}

#[derive(strum::Display, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Usage {
    Session, // web page session token
    Cmdline, // land-cli token
    Worker,  // land-worker token
}

/// get_by_value gets an active token by value
pub async fn get_by_value(value: &str, usage: Option<Usage>) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(value))
        .filter(user_token::Column::Status.eq(Status::Active.to_string()));
    if let Some(u) = usage {
        select = select.filter(user_token::Column::Usage.eq(u.to_string()));
    }
    let token = select.one(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(token)
}

/// list_by_user gets all tokens by user
pub async fn list_by_user(user_id: i32, usage: Option<Usage>) -> Result<Vec<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::UserId.eq(user_id))
        .filter(user_token::Column::Status.eq(Status::Active.to_string()));
    if let Some(u) = usage {
        select = select.filter(user_token::Column::Usage.eq(u.to_string()));
    }
    let tokens = select
        .order_by_desc(user_token::Column::UpdatedAt)
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(tokens)
}

#[derive(Deserialize, Debug)]
pub struct SignCallbackValue {
    pub session_id: String,
    pub avatar_url: String,
    pub first_name: String,
    pub full_name: String,
    pub user_name: String,
    pub email: String,
    pub origin_user_id: String,
    pub origin_provider: String,
}

/// create_session creates a new session token
pub async fn create_session(value: &SignCallbackValue) -> Result<user_token::Model> {
    let mut user = super::user_info::get_by_origin(&value.origin_user_id).await?;
    if user.is_none() {
        let new_user = super::user_info::create(
            value.user_name.clone(),
            value.full_name.clone(),
            value.email.clone(),
            value.avatar_url.clone(),
            value.origin_user_id.clone(),
            value.origin_provider.clone(),
        )
        .await?;
        user = Some(new_user);
    }
    let user = user.unwrap();
    let token_name = format!("session-{}-{}", user.id, chrono::Utc::now().timestamp(),);
    let token = create(user.id, &token_name, 3600 * 23, Usage::Session).await?;
    Ok(token)
}

lazy_static! {
    static ref CREATED_TOKENS: Mutex<HashMap<i32, bool>> = Mutex::new(HashMap::new());
}

/// create a new token
pub async fn create(
    user_id: i32,
    name: &str,
    expire: i64,
    usage: Usage,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now();
    let expired_at = now.add(chrono::TimeDelta::try_seconds(expire).unwrap());
    let value: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    let token_model = user_token::Model {
        id: 0,
        user_id,
        value,
        name: name.to_string(),
        status: Status::Active.to_string(),
        created_at: now.naive_utc(),
        updated_at: now.naive_utc(),
        expired_at: Some(expired_at.naive_utc()),
        deleted_at: None,
        usage: usage.to_string(),
    };
    let mut token_active_model: user_token::ActiveModel = token_model.into();
    token_active_model.id = Default::default();
    let token_model = token_active_model.insert(DB.get().unwrap()).await?;

    // set creating token
    let mut creating_map = CREATED_TOKENS.lock().await;
    creating_map.insert(token_model.id, true);

    Ok(token_model)
}

/// delete deletes a token
pub async fn remove(token_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    user_token::Entity::delete_by_id(token_id).exec(db).await?;
    Ok(())
}

/// is_new checks if a token is new
pub async fn is_new(id: i32) -> bool {
    let creating_map = CREATED_TOKENS.lock().await;
    creating_map.get(&id).is_some()
}

/// unset_new unsets a token as new
pub async fn unset_new(id: i32) {
    let mut creating_map = CREATED_TOKENS.lock().await;
    creating_map.remove(&id);
}
