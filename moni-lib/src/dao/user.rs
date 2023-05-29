use crate::model::user_token;
use crate::{
    db::DB,
    model::user_info::{self, Entity as UserInfoEntity, Model},
};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

enum UserInfoRole {
    // Admin = 1,
    Normal = 2,
}

pub async fn login_by_email(email: String, pwd: String) -> Result<(Model, user_token::Model)> {
    let user = find_by_email(email).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    let user = user.unwrap();
    bcrypt::verify(pwd, &user.password).map_err(|_| anyhow::anyhow!("password not match"))?;

    // create token from webpage, expire in 3 days
    let token = super::token::create(
        user.id as i32,
        String::from("Web Dashboard"),
        String::from("web-dashboard"),
        3 * 24 * 3600,
    )
    .await?;
    Ok((user, token))
}

pub async fn signup_by_email(
    email: String,
    password: String,
    nickname: String,
) -> Result<(Model, user_token::Model)> {
    // if user is exist, return error
    let user = find_by_email(email.clone()).await?;
    if user.is_some() {
        return Err(anyhow::anyhow!("user is exist"));
    }

    // use bcrypt and salt to hash password
    let password_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", password, password_salt);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?.to_string();

    let now = chrono::Utc::now();
    let user_model = user_info::Model {
        id: 0,
        email,
        password,
        password_salt,
        created_at: now,
        updated_at: now,
        display_name: nickname,
        role: UserInfoRole::Normal as i32, // current only support normal user. admin user should not use this api
    };
    let user_active_model: user_info::ActiveModel = user_model.into();
    let db = DB.get().unwrap();
    let user_model = user_active_model.insert(db).await?;

    // create token from webpage, expire in 3 days
    let token = super::token::create(
        user_model.id as i32,
        String::from("Web Dashboard"),
        String::from("web-dashboard"),
        3 * 24 * 3600,
    )
    .await?;
    Ok((user_model, token))
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
