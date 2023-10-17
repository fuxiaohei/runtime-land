use super::RouteError;
use crate::confs;
use axum::{extract::Query, Json};
use hyper::StatusCode;
use land_core::confdata::EndpointConf;
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct ConfValuesMD5Query {
    pub md5: String,
    pub endpoint: String,
}

#[tracing::instrument(name = "[endpoint_get_conf]", skip_all)]
pub async fn conf_handler(
    Query(query): Query<ConfValuesMD5Query>,
    Json(info): Json<land_core::confdata::RuntimeNodeInfo>,
) -> Result<(StatusCode, Json<Option<EndpointConf>>), RouteError> {
    debug!("recv node: {}", info.region_ip());

    confs::runtime_node::update_data(info.clone(), query.md5.clone()).await;

    let conf_values = confs::CONF_VALUES.lock().await;
    if conf_values.md5 == query.md5 {
        return Ok((StatusCode::NOT_MODIFIED, Json(None)));
    }
    debug!(
        "conf_values.md5: {}, query.md5: {}",
        conf_values.md5, query.md5
    );
    Ok((StatusCode::OK, Json(Some(conf_values.clone()))))
}
