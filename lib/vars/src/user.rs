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
