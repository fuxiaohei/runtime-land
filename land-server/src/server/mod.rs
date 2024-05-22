use axum::{middleware, Router};
use land_core_service::httputil::log_middleware;
use std::net::SocketAddr;
use tracing::info;

mod admin;
mod dashboard;
mod examples;
mod templates;
mod workerapi;

/// start the server
pub async fn start(addr: SocketAddr, assets_dir: &str) -> anyhow::Result<()> {
    let dashboard_router = dashboard::router(assets_dir)?;
    let workerapi_router = workerapi::router()?;
    let app = Router::new()
        .nest("/api/v1/worker-api/", workerapi_router)
        .merge(dashboard_router)
        .route_layer(middleware::from_fn(log_middleware));

    info!("Starting server on {}", addr);

    // with connect info
    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    // run it
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
