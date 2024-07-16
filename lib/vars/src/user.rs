use land_dao::{models::user_info, users::UserRole};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AuthUser {
    pub id: i32,
    pub uuid: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub social_name: Option<String>,
    pub social_provider: Option<String>,
    pub social_link: Option<String>,
    pub is_admin: bool,
    pub last_login_at: i64,
    pub created_at: i64,
}

impl AuthUser {
    pub fn new(user: &user_info::Model) -> Self {
        let mut u = AuthUser {
            id: user.id,
            uuid: user.uuid.clone(),
            username: user.name.clone(),
            name: user.nick_name.clone(),
            email: user.email.clone(),
            avatar_url: user.avatar.clone(),
            social_name: None,
            social_provider: None,
            social_link: None,
            is_admin: user.role == UserRole::Admin.to_string(),
            last_login_at: user.last_login_at.and_utc().timestamp(),
            created_at: user.created_at.and_utc().timestamp(),
        };
        if user.oauth_provider.contains("github") {
            u.social_name = Some(user.name.clone());
            u.social_provider = Some("github".to_string());
            u.social_link = Some(format!("https://github.com/{}", user.name));
        }
        u
    }
}
