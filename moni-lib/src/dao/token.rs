use crate::{db::DB, model::user_token};
use anyhow::{Ok, Result};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

pub async fn list(owner_id: i32) -> Result<Vec<user_token::Model>> {
    let db = DB.get().unwrap();
    let tokens = user_token::Entity::find()
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .order_by_desc(user_token::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(tokens)
}

pub async fn find(token: String) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Token.eq(token))
        .one(db)
        .await?;
    Ok(token)
}

pub async fn update_login(token_id: u32) -> Result<()> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now();

    // token must be exist
    let token = user_token::Entity::find()
        .filter(user_token::Column::Id.eq(token_id))
        .one(db)
        .await?
        .unwrap();

    let mut token_model: user_token::ActiveModel = token.into();
    token_model.updated_at = Set(now);
    token_model.update(db).await?;
    Ok(())
}

pub async fn create(
    owner_id: i32,
    name: String,
    origin: String,
    expire: i64,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now();
    let expired_at = now.timestamp() + expire;
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    let uuid = uuid::Uuid::new_v4().to_string();
    let token_model = user_token::Model {
        id: 0,
        owner_id,
        token,
        uuid,
        name,
        created_at: now,
        updated_at: now,
        origin,
        expired_at: expired_at as i32,
    };
    let token_active_model: user_token::ActiveModel = token_model.into();
    let db = DB.get().unwrap();
    let token_model = token_active_model.insert(db).await?;
    Ok(token_model)
}
