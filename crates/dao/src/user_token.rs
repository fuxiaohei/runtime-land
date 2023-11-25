use crate::{model::user_info, model::user_token, DB};
use anyhow::{Ok, Result};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use std::ops::Add;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum CreatedByCases {
    EmailLogin,
    OauthLogin,
    Dashboard,
    Deployment,
    Edgehub,
    ForgetPassword,
    Session,
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

/// find_by_uuid finds a token by uuid
pub async fn find_by_uuid(
    owner_id: i32,
    uuid: String,
    created_by: CreatedByCases,
) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .filter(user_token::Column::Uuid.eq(uuid))
        .filter(user_token::Column::CreatedBy.eq(created_by.to_string()))
        .one(db)
        .await?;
    Ok(token)
}

async fn find_by_value(value: String) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(value))
        .one(db)
        .await?;
    Ok(token)
}

pub async fn refresh(id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    user_token::Entity::update_many()
        .filter(user_token::Column::Id.eq(id))
        .col_expr(
            user_token::Column::UpdatedAt,
            Expr::value(chrono::Utc::now()),
        )
        .exec(db)
        .await?;
    Ok(())
}

pub async fn find_by_value_with_active_user(
    value: String,
) -> Result<(user_token::Model, user_info::Model)> {
    let token = find_by_value(value).await?;
    if token.is_none() {
        return Err(anyhow::anyhow!("token not found"));
    }
    let token = token.unwrap();
    if token.status == Status::Deleted.to_string() {
        return Err(anyhow::anyhow!("token is deleted"));
    }
    let user = super::user::find_by_id(token.owner_id).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    let user = user.unwrap();
    if user.status != super::user::Status::Active.to_string() {
        return Err(anyhow::anyhow!("user status not active"));
    }
    Ok((token, user))
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

pub async fn remove(owner_id: i32, token_uuid: &str) -> Result<()> {
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

/// remove_by_value removes a token by value
pub async fn remove_by_value(token_value: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let token = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(token_value))
        .one(db)
        .await?
        .unwrap();
    let now = chrono::Utc::now();
    let mut token_active_model: user_token::ActiveModel = token.into();
    token_active_model.deleted_at = Set(Some(now));
    token_active_model.status = Set(Status::Deleted.to_string());
    token_active_model.update(db).await?;
    Ok(())
}

/// list_with_page lists tokens with page
pub async fn list_with_page(
    created_by: CreatedByCases,
    page: u64,
    page_size: u64,
) -> Result<(Vec<user_token::Model>, u64, u64)> {
    let db = DB.get().unwrap();
    let pager = user_token::Entity::find()
        .filter(user_token::Column::CreatedBy.eq(created_by.to_string()))
        .filter(user_token::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(user_token::Column::UpdatedAt)
        .paginate(db, page_size);
    let tokens = pager.fetch_page(page - 1).await?;
    let total_pages = pager.num_pages().await?;
    let total_items = pager.num_items().await?;
    Ok((tokens, total_pages, total_items))
}
