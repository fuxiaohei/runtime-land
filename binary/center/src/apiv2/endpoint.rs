use super::RouteError;
use crate::confs;
use axum::{extract::Query, routing::get, Json, Router};
use hyper::StatusCode;
use land_core::confdata::EndpointConf;
use serde::Deserialize;
use tracing::debug;

pub fn router() -> Router {
    Router::new().route("/v2/endpoint/conf", get(get_conf))
}

#[derive(Debug, Deserialize)]
pub struct ConfValuesMD5Query {
    pub md5: String,
    pub endpoint: String,
}

#[tracing::instrument(name = "[endpoint_get_conf]", skip_all)]
async fn get_conf(
    Query(query): Query<ConfValuesMD5Query>,
) -> Result<(StatusCode, Json<Option<EndpointConf>>), RouteError> {
    let conf_values = confs::CONF_VALUES.lock().await;
    if conf_values.md5 == query.md5 {
        debug!("conf_values.md5 == query.md5");
        return Ok((StatusCode::NOT_MODIFIED, Json(None)));
    }
    debug!(
        "conf_values.md5: {}, query.md5: {}",
        conf_values.md5, query.md5
    );
    Ok((StatusCode::OK, Json(Some(conf_values.clone()))))
}
