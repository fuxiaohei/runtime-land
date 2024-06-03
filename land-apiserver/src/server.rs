use anyhow::Result;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use tracing::info;

use crate::v1;

pub async fn start(addr: SocketAddr) -> Result<()> {
    // routes
    let app = Router::new()
        .route("/", get(index))
        .nest("/v1", v1::router()?);

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
