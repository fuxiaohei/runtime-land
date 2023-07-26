use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, Response},
    routing::{any, post},
    Json, Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Mutex};
use tokio::signal;
use tracing::{debug, info};

lazy_static! {
    pub static ref RUNTIMES: Mutex<HashMap<String, RuntimeData>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

async fn default_handler(_req: Request<Body>) -> Response<Body> {
    let builder = Response::builder().status(200);
    builder.body(Body::from("Hello, land-edge")).unwrap()
}

async fn sync_handler(Json(mut payload): Json<RuntimeData>) -> Response<Body> {
    payload.updated_at = chrono::Utc::now().timestamp() as u64;

    debug!("sync_handler begin, payload:{:?}", payload);

    let mut runtimes = RUNTIMES.lock().unwrap();
    runtimes.insert(payload.hostname.clone(), payload);

    let builder = Response::builder().status(200);
    builder.body(Body::from("Hello, sync")).unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeData {
    hostname: String,
    cpu_count: usize,
    cpu_usage: f32,
    total_memory: u64,
    used_memory: u64,
    #[serde(skip_deserializing)]
    updated_at: u64,
}

pub async fn start(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/v1/sync", post(sync_handler))
        .route("/", any(default_handler))
        .route("/*path", any(default_handler));

    info!("Starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutting down");
}
