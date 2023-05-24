use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    let app = Router::new().route("/", get(root));

    info!("starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
