use crate::{
    model::{
        user_info::{self, Model},
        user_token,
    },
    DB,
};
use anyhow::Result;
use gravatar::{Gravatar, Rating};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, FromQueryResult, JsonValue, QueryFilter,
    Statement,
};

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
    let (token, user) = super::user_token::find_by_value_with_active_user(token_value).await?;
    Ok((user, token))
}

/// signup_by_email creates a new user by email and password
/// return user and token
pub async fn signup_by_email(
    email: String,
    pwd: String,
    nickname: String,
) -> Result<(Model, user_token::Model)> {
    let user = find_by_email(email.clone()).await?;
    if user.is_some() {
        return Err(anyhow::anyhow!("user is exist"));
    }
    let password_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", pwd, password_salt);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;
    let avatar = Gravatar::new(&email)
        .set_size(Some(400))
        .set_rating(Some(Rating::Pg))
        .image_url()
        .to_string();

    let now = chrono::Utc::now();
    let user_model = user_info::Model {
        id: 0,
        email,
        password,
        password_salt,
        avatar,
        nick_name: nickname.clone(),
        bio: nickname,
        status: Status::Active.to_string(),
        role: Role::Normal.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        oauth_id: String::new(),
        oauth_provider: String::from("email"),
        oauth_social: String::new(),
    };

    let user_active_model: user_info::ActiveModel = user_model.into();
    let db = DB.get().unwrap();
    let user_model = user_active_model.insert(db).await?;
    // create token from webpage, expire in 10 days
    let token = super::user_token::create(
        user_model.id,
        String::from("signup_by_email"),
        60 * 60 * 24 * 10, // 10 days
        super::user_token::CreatedByCases::EmailLogin,
    )
    .await?;
    Ok((user_model, token))
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

/// get_stats gets the stats of deployments
pub async fn get_stats() -> Result<i32> {
    let db = DB.get().unwrap();
    let values: Vec<JsonValue> = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"select count(id) as counter from user_info where status != 'deleted'"#,
        [],
    ))
    .all(db)
    .await?;
    let counter = values[0]["counter"].as_i64().unwrap() as i32;
    Ok(counter)
}
