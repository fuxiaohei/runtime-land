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
    Set, Statement,
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

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum OauthProvider {
    Email,
    Clerk,
}

async fn find_by_email(email: &str) -> Result<Option<user_info::Model>> {
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
    let user = find_by_email(&email).await?;
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
    let user = find_by_email(&email).await?;
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
        oauth_provider: OauthProvider::Email.to_string(),
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

    // only support clerk oauth provider now
    if oauth_provider != OauthProvider::Clerk.to_string() {
        return Err(anyhow::anyhow!("oauth provider is not supported"));
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

/// forget_password generate a token and send to user's email
pub async fn forget_password(email: &str) -> Result<user_token::Model> {
    let user = find_by_email(email).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("email not found"));
    }
    let user = user.unwrap();
    if user.oauth_provider != OauthProvider::Email.to_string() {
        return Err(anyhow::anyhow!("user is not registered by email"));
    }
    let token = super::user_token::create(
        user.id,
        String::from("forget-password"),
        60 * 60 * 24, // 1 days
        super::user_token::CreatedByCases::ForgetPassword,
    )
    .await?;
    Ok(token)
}

pub async fn update_password(
    id: i32,
    current_password: String,
    new_password: String,
) -> Result<()> {
    let user = find_by_id(id).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("user not found"));
    }
    let user = user.unwrap();
    let full_password = format!("{}{}", current_password, user.password_salt);
    bcrypt::verify(full_password, &user.password)
        .map_err(|_| anyhow::anyhow!("old password not match"))?;

    let password_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", new_password, password_salt);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let mut user_active_model: user_info::ActiveModel = user.into();
    user_active_model.password = Set(password);
    user_active_model.password_salt = Set(password_salt);
    user_active_model.updated_at = Set(chrono::Utc::now());
    let db = DB.get().unwrap();
    user_active_model.update(db).await?;
    Ok(())
}

pub async fn reset_password(token_str: String) -> Result<(String, String)> {
    let (token, user) = crate::user_token::find_by_value_with_active_user(token_str).await?;
    let expire_at = token.expired_at.unwrap().timestamp();
    if expire_at < chrono::Utc::now().timestamp() {
        return Err(anyhow::anyhow!("token expired"));
    }

    let password_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let password_value: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", password_value, password_salt);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let email = user.email.clone();
    let mut user_active_model: user_info::ActiveModel = user.into();
    user_active_model.password = Set(password);
    user_active_model.password_salt = Set(password_salt);
    user_active_model.updated_at = Set(chrono::Utc::now());
    let db = DB.get().unwrap();
    user_active_model.update(db).await?;

    // make token deleted
    let mut token_active_model: user_token::ActiveModel = token.into();
    token_active_model.expired_at = Set(Some(chrono::Utc::now()));
    token_active_model.deleted_at = Set(Some(chrono::Utc::now()));
    token_active_model.status = Set(super::user_token::Status::Deleted.to_string());
    token_active_model.update(db).await?;

    Ok((password_value, email))
}
