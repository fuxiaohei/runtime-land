use land_dao::models::user_token::Model;
use land_dao::DateTimeUTC;
use serde::Serialize;

#[derive(Serialize)]
pub struct TokenVar {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub is_new: bool,
    pub updated_at: DateTimeUTC,
    pub created_at: DateTimeUTC,
}

impl TokenVar {
    pub fn from_models_vec(tokens: Vec<Model>) -> Vec<TokenVar> {
        let mut vars = vec![];
        let now = chrono::Utc::now();
        for token in tokens {
            let mut var = TokenVar {
                id: token.id,
                name: token.name,
                value: String::new(),
                is_new: false,
                updated_at: token.updated_at.and_utc(),
                created_at: token.created_at.and_utc(),
            };
            // if token is created in 30s, it is new
            if now - token.created_at.and_utc() < chrono::Duration::seconds(30) {
                var.is_new = true;
                var.value = token.value;
            }
            vars.push(var);
        }
        vars
    }
}
