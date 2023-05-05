use crate::model::user_token;
use crate::{
    db::DB,
    model::user_info::{self, Entity as UserInfoEntity, Model},
};
use anyhow::Result;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn login_by_email(email: String, pwd: String) -> Result<(Model, user_token::Model)> {
    let user = find_by_email(email).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    let user = user.unwrap();
    if user.password != pwd {
        return Err(anyhow::anyhow!("password not match"));
    }
    let token = super::token::create(
        user.id as i32,
        String::from("Login by Email"),
        String::from("login-by-email"),
        3 * 24 * 3600,
    )
    .await?;
    Ok((user, token))
}

pub async fn login_by_access_token(token_value: String) -> Result<(Model, user_token::Model)> {
    let token_info = super::token::find(token_value).await?;
    if token_info.is_none() {
        return Err(anyhow::anyhow!("token not found"));
    }
    let token_info = token_info.unwrap();
    let user_info = find_by_id(token_info.owner_id).await?;
    if user_info.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    super::token::update_login(token_info.id).await?;
    Ok((user_info.unwrap(), token_info))
}

async fn find_by_id(id: i32) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = UserInfoEntity::find()
        .filter(user_info::Column::Id.eq(id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

async fn find_by_email(email: String) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = UserInfoEntity::find()
        .filter(user_info::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}
