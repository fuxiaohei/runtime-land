use anyhow::{anyhow,Result};
use land_dao::settings;
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;
use std::{collections::HashMap, sync::Once};
use crate::traffic::promql::{flow_ql, projects_flows_ql, request_ql};
use super::{promql::projects_traffic_ql, PeriodParams, Settings, SETTINGS_KEY};

static CLIENT: OnceCell<Client> = OnceCell::new();
static CLIENT_ONCE: Once = Once::new();

async fn traffic_internal(
    period: &PeriodParams,
    query:String) -> Result<MultiLineSeries>{
    let params = Params {
        query: query.clone(),
        step: period.step,
        start: period.start,
        end: period.end,
    };
    let res = range(params).await?;
    let values = LineSeries::from(&res, period.sequence.clone());
    Ok(values)
}

/// requests_traffic queries requests traffic
pub async fn requests_traffic(
    pid: Option<String>,
    uid: Option<String>,
    period: &PeriodParams,
) -> Result<MultiLineSeries> {
    let query = request_ql(pid, uid, &period.step_word)?;
    debug!(
        "query-requests: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    traffic_internal(period, query).await
}

/// projects_traffic queries projects traffic with requests and flows
pub async fn projects_traffic(
    uid:Option<String>,
    pids:Vec<String>,
    period: &PeriodParams,
)-> Result<MultiLineSeries> {
    let query = projects_traffic_ql(uid.clone(), pids.clone(), &period.step_word);
    debug!(
        "query-projects-requests: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    let mut requests = traffic_internal(period, query).await?;
    let query2 = projects_flows_ql(uid, pids,&period.step_word);
    debug!(
        "query-projects-flows: {}, start:{}, end:{}, step:{}",
        query2, period.start, period.end, period.step
    );
    let flows = traffic_internal(period, query2).await?;
    requests.extend(flows);
    Ok(requests)
}

/// flow_traffic queries flows traffic
pub async fn flow_traffic(
    pid: Option<String>,
    uid: Option<String>,
    period: &PeriodParams,
) -> Result<MultiLineSeries> {
    let query = flow_ql(pid, uid, &period.step_word)?;
    debug!(
        "query-flows: {}, start:{}, end:{}, step:{}",
        query, period.start, period.end, period.step
    );
    traffic_internal(period, query).await
}

/// QueryParams is the parameters for querying range
#[derive(Serialize, Debug)]
pub struct Params {
    pub query: String,
    pub start: i64,
    pub end: i64,
    pub step: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status: String,
    pub data: ResponseData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    #[serde(rename = "resultType")]
    pub result_type: String,
    pub result: Vec<ResponseDataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseDataItem {
    pub metric: HashMap<String, String>,
    pub values: Vec<(i64, String)>,
}

/// LineSeries is a series of line chart
#[derive(Debug, Serialize, Deserialize)]
pub struct LineSeries {
    pub total: i64,
    pub values: Vec<(i64, i64)>,
}

impl LineSeries {
    pub fn from(res: &Response, sequence: Vec<i64>) -> MultiLineSeries {
        let mut all_times_data = HashMap::<String, HashMap<i64, i64>>::new();
        for item in res.data.result.iter() {
            let mut times_data = HashMap::<i64, i64>::new();
            // convert metric map to vec and sorted, join by "-" as key
            let mut keys: Vec<String> = item
                .metric
                .iter()
                .map(|(k, v)| format!("{}-{}", k, v))
                .collect();
            keys.sort();
            let mut key = keys.join("-");
            if key.is_empty() {
                key = "metric".to_string();
            }
            for (t, v) in &item.values {
                let value = v.parse::<f64>().unwrap_or(0.0) as i64;
                times_data.insert(*t, value);
            }
            all_times_data.insert(key, times_data);
        }

        let mut res = MultiLineSeries::new();
        for (k, times_data) in all_times_data.iter() {
            let mut values = vec![];
            let mut total: i64 = 0;
            for t in sequence.iter() {
                let value = times_data.get(t).unwrap_or(&0);
                values.push((*t*1000, *value)); // js need timestamp in milliseconds
                total += *value;
            }
            let ls = LineSeries { total, values };
            res.insert(k.clone(), ls);
        }
        res
    }
}

/// MultiLineSeries is a series of line chart
pub type MultiLineSeries = HashMap<String, LineSeries>;

/// query_range queries range from Prometheus
pub async fn range(params: Params) -> Result<Response> {
    CLIENT_ONCE.call_once(|| {
        CLIENT
         .set(Client::new())
         .expect("Failed to create client")
    });

    let prom_env:Settings = settings::get(SETTINGS_KEY).await?.unwrap();
    let client = CLIENT.get().unwrap();
    let resp = client 
        .get(&format!("{}/api/v1/query_range", prom_env.endpoint))
        .query(&params)
        .basic_auth(prom_env.username.clone(), Some(prom_env.password.clone()))
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .send()
        .await?;
    let status = resp.status();
    if status != 200 {
        let text = resp.text().await?;
        return Err(anyhow!("Bad response status: {}, body: {}", status, text));
    }
    let resp = resp.json::<Response>().await?;
    Ok(resp)
}