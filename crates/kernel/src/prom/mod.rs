use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Serialize, Deserialize, Clone)]
pub struct PromEnv {
    pub addr: String,
    pub user: String,
    pub password: String,
}

impl std::fmt::Debug for PromEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromEnv")
            .field("addr", &self.addr)
            .field("user", &self.user)
            .finish()
    }
}

/// PROM_ENV is the environment variables for Prometheus
pub static PROM_ENV: OnceCell<PromEnv> = OnceCell::new();

/// init_prom_env initializes PromEnv from environment variables
pub fn init_prom_env() -> Result<()> {
    let prom_env = PromEnv {
        addr: std::env::var("PROM_ADDR").unwrap_or_default(),
        user: std::env::var("PROM_USER").unwrap_or_default(),
        password: std::env::var("PROM_PASSWORD").unwrap_or_default(),
    };
    info!("PromEnv: {:?}", prom_env);
    PROM_ENV
        .set(prom_env)
        .map_err(|_| anyhow!("PromEnv is already set"))?;
    Ok(())
}

/// QueryRangeParams is the parameters for querying range
#[derive(Serialize)]
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

/// query_range queries range from Prometheus
pub async fn query_range(params: QueryRangeParams) -> Result<QueryResponse> {
    let prom_env = PROM_ENV
        .get()
        .ok_or_else(|| anyhow!("PromEnv is not set"))?;
    let client = reqwest::Client::new();
    // use post to query
    let resp = client
        .post(&format!("{}/api/v1/query_range", prom_env.addr))
        .basic_auth(prom_env.user.clone(), Some(prom_env.password.clone()))
        .form(&params)
        .send()
        .await?;
    let resp = resp.error_for_status()?;
    let resp = resp.json::<QueryResponse>().await?;
    Ok(resp)
}
