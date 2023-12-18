use crate::models::{user_info, user_token};
use crate::DB;
use anyhow::Result;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::ops::Add;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    Normal,
    Admin,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Deleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum CreatedByCases {
    Clerk,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TokenCreatedByCases {
    Session,
    AccessToken,
}

/// find_by_oauth finds a user by oauth user id
pub async fn find_by_oauth(oauth_user_id: &str) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::OauthUserId.eq(oauth_user_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// find_by_id finds a user by id
pub async fn find_by_id(id: i32) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// create creates a new user
pub async fn create(
    name: &str,
    display_name: &str,
    email: &str,
    avatar: &str,
    oauth_user_id: &str,
    oauth_provider: &str,
    oauth_social: &str,
) -> Result<user_info::Model> {
    let user = find_by_oauth(oauth_user_id).await?;
    if user.is_some() {
        return Err(anyhow::anyhow!("user is exist"));
    }
    if oauth_provider != CreatedByCases::Clerk.to_string() {
        return Err(anyhow::anyhow!("oauth provider is not supported"));
    }

    // generate randompassword , and create user
    let password_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", oauth_user_id, password_salt);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let now = chrono::Utc::now();
    let user_model = user_info::Model {
        id: 0,
        email: email.to_string(),
        name: name.to_string(),
        phone: None,
        uuid: uuid::Uuid::new_v4().to_string(),
        password,
        password_salt,
        avatar_url: avatar.to_string(),
        display_name: display_name.to_string(),
        status: Status::Active.to_string(),
        role: Role::Normal.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        created_by: oauth_provider.to_string(),
        oauth_user_id: Some(oauth_user_id.to_string()),
        oauth_social: Some(oauth_social.to_string()),
    };
    let user_active_model: user_info::ActiveModel = user_model.into();
    let db = DB.get().unwrap();
    let user_model = user_active_model.insert(db).await?;
    Ok(user_model)
}

/// create_token creates a new token
pub async fn create_token(
    owner_id: i32,
    name: &str,
    expire: i64,
    created_by: TokenCreatedByCases,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now();
    let expired_at = now.add(chrono::Duration::seconds(expire));
    let value: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    let token_model = user_token::Model {
        id: 0,
        owner_id,
        value,
        name: name.to_string(),
        status: Status::Active.to_string(),
        created_at: now,
        updated_at: now,
        expired_at: Some(expired_at),
        deleted_at: None,
        created_by: created_by.to_string(),
    };
    let token_active_model: user_token::ActiveModel = token_model.into();
    let db = DB.get().unwrap();
    let token_model = token_active_model.insert(db).await?;
    Ok(token_model)
}

/// find_token_by_value finds a token by value
pub async fn find_token_by_value(value: &str) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(value))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(token)
}

/// find_token_by_name finds a token by name
pub async fn find_token_by_name(owner_id: i32, name: &str) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Name.eq(name))
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(token)
}
