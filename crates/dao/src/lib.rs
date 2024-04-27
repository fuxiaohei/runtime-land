pub mod db;
pub mod deployment;
pub mod models;
pub mod projects;
pub mod settings;
pub mod user;
pub mod worker;
pub mod confs;

mod migration;

fn now_time() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

pub type DateTimeUTC = chrono::DateTime<chrono::Utc>;
