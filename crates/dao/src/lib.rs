pub mod db;
pub mod deploy;
pub mod deployment;
pub mod models;
pub mod projects;
pub mod settings;
pub mod user;
pub mod worker;

mod migration;

fn now_time() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

pub type DateTimeUTC = chrono::DateTime<chrono::Utc>;
