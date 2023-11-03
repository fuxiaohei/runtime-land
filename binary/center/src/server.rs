use crate::apiv2;
use crate::pages;
use anyhow::Result;
use axum::extract::DefaultBodyLimit;
use axum::response::{IntoResponse, Redirect};
use axum::{body::Body, http::Request, routing::any, Router};
use std::net::SocketAddr;
use std::process::exit;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;

/// start starts the server.
#[tracing::instrument(name = "[SERVER]", skip_all)]
pub async fn start(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .merge(apiv2::router())
        .merge(pages::router())
        .route("/", any(default_handler))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(TraceLayer::new_for_http());

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// default_handler is the default handler for all requests.
async fn default_handler(_req: Request<Body>) -> impl IntoResponse {
    Redirect::to("/projects").into_response()
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutting down");

    exit(0)
}
