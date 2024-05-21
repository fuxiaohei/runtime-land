use crate::{
    db::DB,
    models::{user_info, user_token},
    now_time,
};
use anyhow::Result;
use lazy_static::lazy_static;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Add;
use tokio::sync::Mutex;

#[derive(strum::Display, PartialEq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TokenUsage {
    Session, // web page session token
    Cmdline, // land-cli token
    Worker,  // land-worker token
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TokenStatus {
    Active,
    Deleted,
}

/// get_token_by_value gets an active token by value
pub async fn get_token_by_value(
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
    let token = select.one(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(token)
}

/// get_token_by_id gets an active token by id
pub async fn get_token_by_id(id: i32) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find_by_id(id)
        .filter(user_token::Column::Status.eq(TokenStatus::Active.to_string()))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(token)
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
pub async fn create_session_token(value: &SignCallbackValue) -> Result<user_token::Model> {
    let mut user = get_info_by_origin_id(&value.origin_user_id).await?;
    if user.is_none() {
        let new_user = create_user(
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
    let token = create_new_token(user.id, &token_name, 3600 * 23, TokenUsage::Session).await?;
    Ok(token)
}

lazy_static! {
    static ref CREATED_TOKENS: Mutex<HashMap<i32, bool>> = Mutex::new(HashMap::new());
}

/// create_new_token a new token
pub async fn create_new_token(
    user_id: i32,
    name: &str,
    expire: i64,
    usage: TokenUsage,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now();
    let expired_at = now.add(chrono::TimeDelta::try_seconds(expire).unwrap());
    let value: String = land_common::encoding::rand_string(40);
    let token_model = user_token::Model {
        id: 0,
        user_id,
        value,
        name: name.to_string(),
        status: TokenStatus::Active.to_string(),
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

/// is_new_token checks if a token is new
pub async fn is_new_token(id: i32) -> bool {
    let creating_map = CREATED_TOKENS.lock().await;
    creating_map.get(&id).is_some()
}

/// unset_new_token unsets a token as new
pub async fn unset_new_token(id: i32) {
    let mut creating_map = CREATED_TOKENS.lock().await;
    creating_map.remove(&id);
}

/// list_tokens_by_user gets all tokens by user
pub async fn list_tokens_by_user(
    user_id: i32,
    usage: Option<TokenUsage>,
) -> Result<Vec<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::UserId.eq(user_id))
        .filter(user_token::Column::Status.eq(TokenStatus::Active.to_string()));
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

/// remove_token deletes a token
pub async fn remove_token(token_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    user_token::Entity::delete_by_id(token_id).exec(db).await?;
    Ok(())
}

/// remove_session_token removes a session token
pub async fn remove_session_token(value: &str) -> Result<()> {
    let token = get_token_by_value(value, Some(TokenUsage::Session)).await?;
    if let Some(token) = token {
        remove_token(token.id).await?;
    }
    Ok(())
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum UserStatus {
    Active,
    Disabled,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum UserRole {
    Normal,
    Admin,
}

/// get_info_by_id finds a user by id
pub async fn get_info_by_id(
    id: i32,
    status: Option<UserStatus>,
) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_info::Entity::find_by_id(id);
    if let Some(s) = status {
        select = select.filter(user_info::Column::Status.eq(s.to_string()));
    }
    let user = select.one(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// get_info_by_origin_id returns a user by origin_user_id
pub async fn get_info_by_origin_id(origin_user_id: &str) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::OriginUserId.eq(origin_user_id))
        .filter(user_info::Column::Status.eq(UserStatus::Active.to_string()))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// create_user creates a new user
pub async fn create_user(
    name: String,
    nick_name: String,
    email: String,
    gravatar: String,
    origin_user_id: String,
    origin_provider: String,
) -> Result<user_info::Model> {
    // currently must be clerk-xxx
    if !origin_provider.starts_with("clerk@") {
        return Err(anyhow::anyhow!("OAuth provider is not supported"));
    }
    // generate randompassword , and create user
    let password_salt = land_common::encoding::rand_string(20);
    let full_password = format!("{}{}", password_salt, origin_user_id);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let now = now_time();
    let uuid = uuid::Uuid::new_v4().to_string();
    let user_model = user_info::Model {
        id: 0,
        uuid,
        email,
        name,
        password,
        password_salt,
        gravatar,
        nick_name,
        status: UserStatus::Active.to_string(),
        role: UserRole::Normal.to_string(),
        created_at: now,
        last_login_at: now,
        updated_at: now,
        deleted_at: None,
        origin_user_id: Some(origin_user_id),
        origin_email_id: None,
        origin_provider,
    };
    let mut user_active_model: user_info::ActiveModel = user_model.into();
    user_active_model.id = Default::default();
    let user_model = user_active_model.insert(DB.get().unwrap()).await?;
    Ok(user_model)
}

/// disable_user disables a user
pub async fn disable_user(user_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    let now = now_time();
    user_info::Entity::update_many()
        .col_expr(
            user_info::Column::Status,
            Expr::value(UserStatus::Disabled.to_string()),
        )
        .col_expr(user_info::Column::UpdatedAt, Expr::value(now))
        .filter(user_info::Column::Id.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

/// list_infos lists user infos by ids
pub async fn list_infos(user_ids: Vec<i32>) -> Result<HashMap<i32, user_info::Model>> {
    let db = DB.get().unwrap();
    let users = user_info::Entity::find()
        .filter(user_info::Column::Id.is_in(user_ids))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let mut user_map = HashMap::new();
    for user in users {
        user_map.insert(user.id, user);
    }
    Ok(user_map)
}
