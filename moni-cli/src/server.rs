use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use futures_util::StreamExt;
use std::net::SocketAddr;
use tracing::info;

/// start server
pub async fn start(addr: SocketAddr) -> Result<()> {
    // build our application with default handler
    let app = Router::new()
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn default_handler(mut req: Request<Body>) -> Response<Body> {
    while let Some(chunk) = req.body_mut().next().await {
        println!("----- chunk = {:?}", chunk.unwrap().len());
    }
    Response::builder()
        .status(200)
        .body(Body::from("Hello, World!"))
        .unwrap()
}
