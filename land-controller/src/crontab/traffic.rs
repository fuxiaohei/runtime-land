use anyhow::Result;
use land_core_service::metrics::traffic::{refresh_projects, refresh_total};
use tracing::{info, instrument};

/// refresh refreshes the metrics
#[instrument("[TRAFFIC]")]
pub async fn refresh() -> Result<()> {
    info!("refresh");
    let (projects, _) = land_dao::projects::list_paginate(1, 10000).await?;
    let mut pids = vec![];
    let mut pids2 = vec![];
    for p in projects {
        pids.push((p.id, p.uuid));
        pids2.push(p.id);
    }
    refresh_projects(pids).await?;
    refresh_total(pids2).await?;
    Ok(())
}
