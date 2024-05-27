use anyhow::Result;
use serde::Serialize;

/// DashboardVars is the vars for admin dashboard page
#[derive(Serialize)]
pub struct DashboardVars {
    pub user_count: u64,
    pub project_count: u64,
    pub total_requests: i32,
    pub total_bytes: i32,
}

impl DashboardVars {
    pub async fn new() -> Result<Self> {
        let user_count = land_dao::user::count().await?;
        let project_count = land_dao::projects::count().await?;
        let traffic_summary = land_dao::traffic::get_current_total().await?;
        Ok(Self {
            user_count,
            project_count,
            total_requests: traffic_summary.as_ref().map_or(0, |s| s.requests),
            total_bytes: traffic_summary.as_ref().map_or(0, |s| s.transferred_bytes),
        })
    }
}
