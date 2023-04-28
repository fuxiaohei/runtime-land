use crate::{
    db::DB,
    model::{
        user_info::{self, Entity as UserInfoEntity, Model},
        user_token,
    },
};
use anyhow::Result;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

pub async fn login_by_email(email: String, pwd: String) -> Result<(Model, String)> {
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
    let token = create_token(
        user.id as i32,
        String::from("Login by Email"),
        String::from("login-by-email"),
        3 * 24 * 3600,
    )
    .await?;
    Ok((user, token))
}

async fn create_token(owner_id: i32, name: String, origin: String, expire: i64) -> Result<String> {
    let now = chrono::Utc::now();
    let expired_at = now.timestamp() + expire;
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    let token_model = user_token::Model {
        id: 0,
        owner_id,
        token,
        name,
        created_at: now,
        updated_at: now,
        origin,
        expired_at: expired_at as i32,
    };
    let token_active_model: user_token::ActiveModel = token_model.into();
    let db = DB.get().unwrap();
    let token_model = token_active_model.insert(db).await?;
    Ok(token_model.token)
}
