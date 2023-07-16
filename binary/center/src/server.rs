use crate::restapi;
use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;

/// start starts the server.
pub async fn start(addr: SocketAddr) -> Result<()> {
    let app = restapi::router();

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
