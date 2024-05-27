use crate::models::project_traffic;
use crate::{db::DB, now_time};
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Default)]
pub struct TrafficSummary {
    pub requests: i32,
    pub transferred_bytes: i32,
}

/// summary_projects_traffic lists projects traffic data
pub async fn summary_projects_traffic(projects: Vec<i32>) -> Result<HashMap<i32, TrafficSummary>> {
    let mut m: HashMap<i32, TrafficSummary> = HashMap::new();
    let (_, current_hour_str) = get_traffic_hour(0);
    let (_, prev_hour_str) = get_traffic_hour(-1);
    let db = DB.get().unwrap();
    let results = project_traffic::Entity::find()
        .filter(project_traffic::Column::ProjectId.is_in(projects))
        .filter(project_traffic::Column::TrafficKey.is_in(vec!["requests", "transferred_bytes"]))
        .filter(project_traffic::Column::TimeAt.is_in(vec![current_hour_str, prev_hour_str]))
        .order_by_asc(project_traffic::Column::Id)
        .all(db)
        .await?;
    for r in results {
        let summary = m.entry(r.project_id).or_insert(TrafficSummary {
            requests: 0,
            transferred_bytes: 0,
        });
        if r.traffic_key == "requests" {
            summary.requests = r.value;
        } else if r.traffic_key == "transferred_bytes" {
            summary.transferred_bytes = r.value;
        }
    }
    Ok(m)
}

/// get_traffic gets the traffic summary of a project
pub async fn get_traffic(project_id: i32, diff: i64) -> Result<Option<TrafficSummary>> {
    let (_, time_at) = get_traffic_hour(diff);
    let db = DB.get().unwrap();
    let results = project_traffic::Entity::find()
        .filter(project_traffic::Column::ProjectId.eq(project_id))
        .filter(project_traffic::Column::TrafficKey.is_in(vec!["requests", "transferred_bytes"]))
        .filter(project_traffic::Column::TimeAt.eq(time_at))
        .all(db)
        .await?;
    if results.len() != 2 {
        return Ok(None);
    }
    let mut summary = TrafficSummary {
        requests: 0,
        transferred_bytes: 0,
    };
    for r in results {
        if r.traffic_key == "requests" {
            summary.requests = r.value;
        } else if r.traffic_key == "transferred_bytes" {
            summary.transferred_bytes = r.value;
        }
    }
    Ok(Some(summary))
}

/// get_current_total gets the current total traffic summary
pub async fn get_current_total() -> Result<Option<TrafficSummary>> {
    let summary = get_traffic(i32::MAX - 1, 0).await?;
    if summary.is_none() {
        return get_traffic(i32::MAX - 1, -1).await;
    }
    Ok(summary)
}

/// save_traffic saves the traffic summary of a project
pub async fn save_traffic(
    project_id: i32,
    time_at: String,
    requests: i32,
    transferred_bytes: i32,
) -> Result<()> {
    let db = DB.get().unwrap();
    let requests = project_traffic::Model {
        id: 0,
        project_id,
        time_at: time_at.clone(),
        traffic_key: "requests".to_string(),
        value: requests,
        created_at: now_time(),
    };
    let mut requests_active_model = requests.into_active_model();
    requests_active_model.id = Default::default();
    requests_active_model.insert(db).await?;

    let transferred_bytes = project_traffic::Model {
        id: 0,
        project_id,
        time_at,
        traffic_key: "transferred_bytes".to_string(),
        value: transferred_bytes,
        created_at: now_time(),
    };
    let mut transferred_bytes_active_model = transferred_bytes.into_active_model();
    transferred_bytes_active_model.id = Default::default();
    transferred_bytes_active_model.insert(db).await?;
    Ok(())
}

/// get_traffic_hour gets the traffic time stamp of the current hour
pub fn get_traffic_hour(diff: i64) -> (i64, String) {
    // get current time hour timestamp
    let current_timestamp = chrono::Utc::now().timestamp() + 3600 * diff;
    let current_hour_ts = current_timestamp - current_timestamp % 3600;
    // format current_hour_ts to human readable
    let current_hour = chrono::DateTime::from_timestamp(current_hour_ts, 0).unwrap();
    let current_hour_str = current_hour.format("%Y%m%d%H%M").to_string();
    (current_hour_ts, current_hour_str)
}
