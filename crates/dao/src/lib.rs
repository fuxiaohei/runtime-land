pub mod confs;
pub mod db;
pub mod deployment;
pub mod envs;
pub mod metrics;
pub mod models;
pub mod projects;
pub mod settings;
pub mod traffic;
pub mod user;
pub mod worker;

mod migration;

fn now_time() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

pub type DateTimeUTC = chrono::DateTime<chrono::Utc>;
