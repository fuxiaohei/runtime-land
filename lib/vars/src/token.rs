use land_dao::models::user_token;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub value: String,
    pub name: String,
    pub created_at: i64,
    pub latest_used_at: i64,
    pub expired_at: i64,
    pub is_new: bool,
    pub id: i32,
}

impl Token {
    pub fn new(m: user_token::Model) -> Self {
        let expired_at = if let Some(expired_at) = m.expired_at {
            expired_at.and_utc().timestamp()
        } else {
            0
        };
        let now = chrono::Utc::now().timestamp();
        let is_new = m.created_at.and_utc().timestamp() + 30 > now;
        let mut token = Token {
            value: String::new(),
            name: m.name,
            created_at: m.created_at.and_utc().timestamp(),
            latest_used_at: m.latest_used_at.and_utc().timestamp(),
            expired_at,
            id: m.id,
            is_new,
        };
        if is_new {
            token.value = m.value;
        }
        token
    }
    pub fn new_from_models(models: Vec<user_token::Model>) -> Vec<Self> {
        models.into_iter().map(Token::new).collect()
    }
}
