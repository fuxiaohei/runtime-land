use anyhow::Result;
use land_core::metrics::MultiLineSeries;
use tracing::{debug, info};

/// refresh refreshes the metrics
pub async fn refresh() -> Result<()> {
    info!("Traffic::refresh");

    let (current_hour_ts, current_hour_str) = land_dao::projects::get_traffic_hour();
    let (projects, _) = land_dao::projects::list_paginate(1, 10000).await?;
    for p in projects {
        let traffic_data = land_dao::projects::get_traffic(p.id).await?;
        if traffic_data.is_some() {
            continue;
        }
        debug!(project_id = p.id, "Traffic refresh");

        let period = TrafficPeriodParams::new("1d", Some(current_hour_ts));

        // get requests total value
        let requests_data =
            query_requests_traffic(String::new(), Some(p.uuid.clone()), &period).await?;
        let requests_value = if requests_data.is_empty() {
            0
        } else {
            let series = requests_data.get("metric").unwrap();
            series.total
        };

        // get flows value
        let flows_data = query_flows_traffic(String::new(), Some(p.uuid), &period).await?;
        let flows_value = if flows_data.is_empty() {
            0
        } else {
            let mut total = 0;
            for (_, series) in flows_data.iter() {
                if series.total > 0 {
                    total += series.total;
                }
            }
            total
        };
        land_dao::projects::save_traffic(
            p.id,
            current_hour_str.clone(),
            requests_value as i32,
            flows_value as i32,
        )
        .await?;
        debug!(
            project_id = p.id,
            "Traffic refresh done, requests: {}, flows: {}", requests_value, flows_value,
        );
    }

    Ok(())
}

#[derive(Debug)]
pub struct TrafficPeriodParams {
    pub start: i64,
    pub end: i64,
    pub step: i64,
    pub step_word: String,
    pub sequence: Vec<i64>, // unix timestamp from start to end with step
}

impl TrafficPeriodParams {
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
            return TrafficPeriodParams {
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
        TrafficPeriodParams {
            start,
            end,
            step: 600,
            step_word: "10m".to_string(),
            sequence,
        }
    }
}

fn get_request_query(acc: &str, project: Option<String>, step: &str) -> String {
    if let Some(pid) = project {
        "sum(increase(req_fn_total{p_uuid='".to_string() + &pid + "',status='all'}[" + step + "]))"
    } else {
        "sum(increase(req_fn_total{uuid='".to_string() + acc + "',status='all'}[" + step + "]))"
    }
}

/// query_requests_traffic queries requests traffic
pub async fn query_requests_traffic(
    acc: String,
    project: Option<String>,
    period: &TrafficPeriodParams,
) -> Result<MultiLineSeries> {
    let query = get_request_query(&acc, project, &period.step_word);
    debug!(
        "query: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    let params = land_core::metrics::QueryRangeParams {
        query: query.clone(),
        step: period.step,
        start: period.start,
        end: period.end,
    };
    let res = land_core::metrics::query_range(params).await?;
    let values = land_core::metrics::LineSeries::from(&res, period.sequence.clone());
    Ok(values)
}

fn get_flow_query(acc: String, project: Option<String>, step: &str) -> String {
    if let Some(pid) = project {
        "sum by (p_uuid,flowtype) (increase(req_fn_bytes_total{p_uuid='".to_string()
            + &pid
            + "'}["
            + step
            + "]))"
    } else {
        "sum by (uuid,flowtype) (increase(req_fn_bytes_total{uuid='".to_string()
            + &acc
            + "'}["
            + step
            + "]))"
    }
}

pub async fn query_flows_traffic(
    acc: String,
    project: Option<String>,
    period: &TrafficPeriodParams,
) -> Result<MultiLineSeries> {
    let query = get_flow_query(acc, project, &period.step_word);
    // end time is now ts with latest 10 decade
    debug!(
        "query: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    let params = land_core::metrics::QueryRangeParams {
        query: query.clone(),
        step: period.step,
        start: period.start,
        end: period.end,
    };
    let res = land_core::metrics::query_range(params).await?;
    let values = land_core::metrics::LineSeries::from(&res, period.sequence.clone());
    Ok(values)
}
