use std::ops::Add;

use crate::{model::user_token, DB};
use anyhow::{Ok, Result};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum CreatedByCases {
    EmailLogin,
    OauthLogin,
    Dashboard,
    Deployment,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    InActive,
    Expired,
    Deleted,
}

pub async fn list_by_created(
    owner_id: i32,
    created_by: CreatedByCases,
) -> Result<Vec<user_token::Model>> {
    let db = DB.get().unwrap();
    let tokens = user_token::Entity::find()
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .filter(user_token::Column::CreatedBy.eq(created_by.to_string()))
        .filter(user_token::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(user_token::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(tokens)
}

// find_by_name finds a token by name
pub async fn find_by_name(
    owner_id: i32,
    name: String,
    created_by: CreatedByCases,
) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .filter(user_token::Column::Name.eq(name))
        .filter(user_token::Column::CreatedBy.eq(created_by.to_string()))
        .one(db)
        .await?;
    Ok(token)
}

pub async fn find(value: String) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(value))
        .one(db)
        .await?;
    Ok(token)
}

pub async fn create(
    owner_id: i32,
    name: String,
    expire: i64,
    created_by: CreatedByCases,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now();
    let expired_at = now.add(chrono::Duration::seconds(expire));
    let value: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    let uuid = uuid::Uuid::new_v4().to_string();
    let token_model = user_token::Model {
        id: 0,
        owner_id,
        value,
        uuid,
        name,
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

pub async fn remove(owner_id: i32, token_uuid: String) -> Result<()> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Uuid.eq(token_uuid))
        .one(db)
        .await?
        .unwrap();
    if token.owner_id != owner_id {
        return Err(anyhow::anyhow!("token uuid and owner id not match"));
    }
    let now = chrono::Utc::now();
    let mut token_active_model: user_token::ActiveModel = token.into();
    token_active_model.deleted_at = Set(Some(now));
    token_active_model.status = Set(Status::Deleted.to_string());
    token_active_model.update(db).await?;
    Ok(())
}
