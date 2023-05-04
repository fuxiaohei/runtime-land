use crate::model::user_token;
use crate::{
    db::DB,
    model::user_info::{self, Entity as UserInfoEntity, Model},
};
use anyhow::Result;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn login_by_email(email: String, pwd: String) -> Result<(Model, user_token::Model)> {
    let db = DB.get().unwrap();
    let user = UserInfoEntity::find()
        .filter(user_info::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
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
