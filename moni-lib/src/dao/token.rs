use crate::{db::DB, model::user_token};
use anyhow::Result;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn list(owner_id: i32) -> Result<Vec<user_token::Model>> {
    let db = DB.get().unwrap();
    let tokens = user_token::Entity::find()
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .all(db)
        .await?;
    Ok(tokens)
}
