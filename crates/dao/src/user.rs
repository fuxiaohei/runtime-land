use crate::{
    model::{
        user_info::{self, Model},
        user_token,
    },
    DB,
};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

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

async fn find_by_email(email: String) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

pub async fn find_by_oauth_id(oauth_id: String) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::OauthId.eq(oauth_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

pub async fn find_by_id(id: i32) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::Id.eq(id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

pub async fn login_by_email(email: String, pwd: String) -> Result<(Model, user_token::Model)> {
    let user = find_by_email(email).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    let user = user.unwrap();
    bcrypt::verify(pwd, &user.password).map_err(|_| anyhow::anyhow!("password not match"))?;

    // create token from webpage, expire in 3 days
    let token = super::user_token::create(
        user.id,
        String::from("login-by-email"),
        60 * 60 * 24 * 3, // 3 days
        super::user_token::CreatedByCases::EmailLogin,
    )
    .await?;
    Ok((user, token))
}

pub async fn login_by_token(token_value: String) -> Result<(Model, user_token::Model)> {
    let token_info = super::user_token::find_by_value(token_value).await?;
    if token_info.is_none() {
        return Err(anyhow::anyhow!("token not found"));
    }
    let token_info = token_info.unwrap();
    let user_info = find_by_id(token_info.owner_id).await?;
    if user_info.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    // super::user_token::update_login(token_info.id).await?;
    Ok((user_info.unwrap(), token_info))
}

pub async fn signup_by_oauth(
    name: String,
    display_name: String,
    email: String,
    avatar: String,
    oauth_id: String,
    oauth_provider: String,
    oauth_social: String,
) -> Result<(Model, user_token::Model)> {
    let user = find_by_oauth_id(oauth_id.clone()).await?;
    if user.is_some() {
        return Err(anyhow::anyhow!("user is exist"));
    }

    // use bcrypt and salt to hash password
    let password_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", oauth_id, password_salt);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let now = chrono::Utc::now();
    let user_model = user_info::Model {
        id: 0,
        email,
        password,
        password_salt,
        avatar,
        nick_name: display_name,
        bio: name,
        status: Status::Active.to_string(),
        role: Role::Normal.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        oauth_id,
        oauth_provider,
        oauth_social,
    };
    let user_active_model: user_info::ActiveModel = user_model.into();
    let db = DB.get().unwrap();
    let user_model = user_active_model.insert(db).await?;
    // create token from webpage, expire in 10 days
    let token = super::user_token::create(
        user_model.id,
        String::from("signup_by_oauth"),
        60 * 60 * 24 * 10, // 10 days
        super::user_token::CreatedByCases::EmailLogin,
    )
    .await?;
    Ok((user_model, token))
}
