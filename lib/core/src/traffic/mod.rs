use anyhow::Result;
use land_dao::settings;
use serde::{Deserialize, Serialize};

mod promql;
mod query;
pub use query::{flow_traffic, projects_traffic, requests_traffic};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Settings {
    pub endpoint: String,
    pub username: String,
    pub password: String,
}

static SETTINGS_KEY: &str = "prometheus-settings";

/// get_settings get settings from db
pub async fn get_settings() -> Result<Settings> {
    let s: Option<Settings> = settings::get(SETTINGS_KEY).await?;
    Ok(s.unwrap_or_default())
}

/// set_settings set settings to db
pub async fn set_settings(settings: Settings) -> Result<()> {
    settings::set(SETTINGS_KEY, settings).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Form {
    pub period: String,
    pub pid: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectsQueryForm {
    pub period: String,
    pub pids: Vec<i32>,
}

#[derive(Debug)]
pub struct PeriodParams {
    pub start: i64,
    pub end: i64,
    pub step: i64,
    pub step_word: String,
    pub sequence: Vec<i64>, // unix timestamp from start to end with step
}

impl PeriodParams {
    pub fn new(period: &str, start_ts: Option<i64>) -> Self {
        let st = if let Some(t) = start_ts {
            t
        } else {
            chrono::Utc::now().timestamp()
        };
        if period == "7d" {
            let end = (st + 3599) / 3600 * 3600; // add 3500 for round hour, use next hour
            let start = end - 604800; // 86400 * 7 + 2
            let sequence = (0..85).map(|i| start + i * 3600 * 2).collect();
            return Self {
                start,
                end,
                step: 3600,
                step_word: "2h".to_string(),
                sequence,
            };
        }
        let end = (st + 599) / 600 * 600; // use next 10 min
        let start = end - 86400; // oneday 1440/10+1
        let sequence = (0..145).map(|i| start + i * 600).collect();
        Self {
            start,
            end,
            step: 600,
            step_word: "10m".to_string(),
            sequence,
        }
    }
}
