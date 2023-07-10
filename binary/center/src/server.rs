use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use std::net::SocketAddr;
use tracing::info;

/// default_handler is the default handler for all requests.
async fn default_handler(_req: Request<Body>) -> Response<Body> {
    let builder = Response::builder().status(200);
    builder.body(Body::from("Hello, land-center")).unwrap()
}

/// start starts the server.
pub async fn start(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
