use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

/// QueryRangeParams is the parameters for querying range
#[derive(Serialize, Debug)]
pub struct QueryRangeParams {
    pub query: String,
    pub start: i64,
    pub end: i64,
    pub step: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub status: String,
    pub data: QueryResponseData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponseData {
    #[serde(rename = "resultType")]
    pub result_type: String,
    pub result: Vec<QueryResponseDataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponseDataItem {
    pub metric: HashMap<String, String>,
    pub values: Vec<(i64, String)>,
}

/// LineSeries is a series of line chart
#[derive(Debug, Serialize, Deserialize)]
pub struct LineSeries {
    pub total: i64,
    pub values: Vec<(i64, i64)>,
}

/// MultiLineSeries is a series of line chart
pub type MultiLineSeries = HashMap<String, LineSeries>;

impl LineSeries {
    pub fn from(res: &QueryResponse, sequence: Vec<i64>) -> MultiLineSeries {
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
                values.push((*t * 1000, *value)); // js use milliseconds, t*1000
                total += *value;
            }
            let ls = LineSeries { total, values };
            res.insert(k.clone(), ls);
        }
        res
    }
}

/// query_range queries range from Prometheus
pub async fn query_range(params: QueryRangeParams) -> Result<QueryResponse> {
    let prom_env = super::get_env().await?;
    let resp = CLIENT 
        .get(&format!("{}/api/v1/query_range", prom_env.addr))
        .query(&params)
        .basic_auth(prom_env.user.clone(), Some(prom_env.password.clone()))
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .send()
        .await?;
    let status = resp.status();
    if status != 200 {
        let text = resp.text().await?;
        return Err(anyhow!("Bad response status: {}, body: {}", status, text));
    }
    let resp = resp.json::<QueryResponse>().await?;
    Ok(resp)
}
