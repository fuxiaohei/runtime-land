pub mod db;
pub mod models;
pub mod projects;
pub mod settings;
pub mod user;
pub mod deploy;

mod migration;

fn now_time() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

pub type DateTimeUTC = chrono::DateTime<chrono::Utc>;
