use std::collections::HashMap;
use crate::{models::user_info, now_time, DB};
use anyhow::{anyhow, Result};
use land_common::rand_string;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

/// ----- user

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

/// get_by_id finds a user by id
pub async fn get_by_id(id: i32, status: Option<UserStatus>) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_info::Entity::find_by_id(id);
    if let Some(s) = status {
        select = select.filter(user_info::Column::Status.eq(s.to_string()));
    }
    let user = select.one(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// get_by_oauth_id returns a user by oauth_id
pub async fn get_by_oauth_id(oauth_id: &str) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::OauthUserId.eq(oauth_id))
        .filter(user_info::Column::Status.eq(UserStatus::Active.to_string()))
        .one(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(user)
}

/// find_by_ids returns a list of users by ids
pub async fn find_by_ids(ids: Vec<i32>) -> Result<HashMap<i32, user_info::Model>> {
    let db = DB.get().unwrap();
    let users = user_info::Entity::find()
        .filter(user_info::Column::Id.is_in(ids))
        .all(db)
        .await
        .map_err(|e| anyhow!(e))?;
    let mut map = HashMap::new();
    for user in users {
        map.insert(user.id, user);
    }
    Ok(map)
}

/// is_first checks if the system has the first user
pub async fn is_first() -> Result<bool> {
    let db = DB.get().unwrap();
    let count = user_info::Entity::find()
        .count(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(count < 1)
}

/// create creates a new user
pub async fn create(
    name: String,
    nick_name: String,
    email: String,
    avatar: String,
    oauth_user_id: String,
    oauth_provider: String,
    user_role: Option<UserRole>,
) -> Result<user_info::Model> {
    // currently must be clerk-xxx
    if !oauth_provider.starts_with("clerk@") {
        return Err(anyhow::anyhow!("OAuth provider is not supported"));
    }
    // generate randompassword , and create user
    let password_salt = rand_string(20);
    let full_password = format!("{}{}", password_salt, oauth_user_id);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    // role is optional, default is normal
    let role = user_role.unwrap_or(UserRole::Normal).to_string();

    let now = now_time();
    let uuid = uuid::Uuid::new_v4().to_string();
    let user_model = user_info::Model {
        id: 0,
        uuid,
        email,
        name,
        password,
        password_salt,
        avatar,
        nick_name,
        status: UserStatus::Active.to_string(),
        role,
        created_at: now,
        last_login_at: now,
        updated_at: now,
        deleted_at: None,
        oauth_user_id: Some(oauth_user_id),
        oauth_email_id: None,
        oauth_provider,
    };
    let mut user_active_model: user_info::ActiveModel = user_model.into();
    user_active_model.id = Default::default();
    let user_model = user_active_model.insert(DB.get().unwrap()).await?;
    Ok(user_model)
}
