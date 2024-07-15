use crate::{admin, dash, worker_api};
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{debug, info};

/// start starts the server.
pub async fn start(addr: SocketAddr, assets_dir: &str, tpl_dir: Option<String>) -> Result<()> {
    let dash_routes = dash::route(assets_dir, tpl_dir).await?;
    let worker_api_routes = worker_api::route().await?;
    let admin_routes = admin::route().await?;

    let app = Router::new()
        .merge(dash_routes)
        .nest("/worker-api", worker_api_routes)
        .nest("/admin", admin_routes);
    // with connect info
    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    info!("Listening on {}", addr);

    // run http server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            debug!("Ctrl-C received, shutting down");
        },
        _ = terminate => {
            debug!("SIGTERM received, shutting down");
        },
    }
}
