use axum::routing::get;
use axum::{middleware, Router};
use axum_csrf::{CsrfConfig, CsrfLayer};
use land_core_service::clerkauth::admin_middleware;
use land_core_service::httputil::log_middleware;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::info;

mod index;
mod projects;
mod templates;

/// start the server
pub async fn start(addr: SocketAddr, assets_dir: &str) -> anyhow::Result<()> {
    // init templates
    templates::extract(assets_dir)?;
    let hbs = land_core_service::template::init(assets_dir)?;
    let static_assets_dir = format!("{}/static", assets_dir);
    let config = CsrfConfig::default();

    let app = Router::new()
        .route("/", get(index::index))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .layer(CsrfLayer::new(config))
        .with_state(axum_template::engine::Engine::from(hbs))
        .route_layer(middleware::from_fn(admin_middleware))
        .route_layer(middleware::from_fn(log_middleware));

    info!("Starting server on {}", addr);

    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    // run it
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
