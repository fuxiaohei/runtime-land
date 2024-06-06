use anyhow::Result;
use axum::routing::get;
use axum::{middleware, Router};
use land_core_service::httputil::log_middleware;
use std::net::SocketAddr;
use tracing::info;

use crate::v1;

pub async fn start(addr: SocketAddr) -> Result<()> {
    // routes
    let app = Router::new()
        .route("/", get(index))
        .merge(v1::router()?)
        .route_layer(middleware::from_fn(log_middleware));

    info!("Starting server on {}", addr);

    // with connect info
    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    // run server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> &'static str {
    "Hello, World!"
}
