use land_dao::DateTimeUTC;
use serde::Serialize;

#[derive(Serialize)]
pub struct TokenVar {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub is_new: bool,
    pub updated_at: DateTimeUTC,
}
