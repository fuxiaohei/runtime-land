use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::{models::user_info, now_time, DB};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    SelfDeleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    Normal,
    Admin,
}

/// get_by_id finds a user by id
pub async fn get_by_id(id: i32, status: Option<Status>) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_info::Entity::find_by_id(id);
    if let Some(s) = status {
        select = select.filter(user_info::Column::Status.eq(s.to_string()));
    }
    let user = select.one(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// get_by_origin returns a user by origin_user_id
pub async fn get_by_origin(origin_user_id: &str) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let user = user_info::Entity::find()
        .filter(user_info::Column::OriginUserId.eq(origin_user_id))
        .filter(user_info::Column::Status.eq(Status::Active.to_string()))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(user)
}

/// create creates a new user
pub async fn create(
    name: String,
    nick_name: String,
    email: String,
    gravatar: String,
    origin_user_id: String,
    origin_provider: String,
) -> Result<user_info::Model> {
    // currently must be clerk-xxx
    if !origin_provider.starts_with("clerk@") {
        return Err(anyhow::anyhow!("OAuth provider is not supported"));
    }
    // generate randompassword , and create user
    let password_salt: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    let full_password = format!("{}{}", password_salt, origin_user_id);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let now = now_time();
    let user_model = user_info::Model {
        id: 0,
        email,
        name,
        password,
        password_salt,
        gravatar,
        nick_name,
        status: Status::Active.to_string(),
        role: Role::Normal.to_string(),
        created_at: now,
        last_login_at: now,
        updated_at: now,
        deleted_at: None,
        origin_user_id: Some(origin_user_id),
        origin_email_id: None,
        origin_provider,
    };
    let mut user_active_model: user_info::ActiveModel = user_model.into();
    user_active_model.id = Default::default();
    let user_model = user_active_model.insert(DB.get().unwrap()).await?;
    Ok(user_model)
}
