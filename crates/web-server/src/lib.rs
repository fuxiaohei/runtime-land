use anyhow::Result;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

/// router returns api server router
pub fn router() -> Router {
    Router::new().route("/", get(root))
}

/// run starts api server
pub async fn run(addr: SocketAddr) -> Result<()> {
    let app = router();

    info!("Starting on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
