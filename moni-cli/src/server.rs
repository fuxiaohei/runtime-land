use anyhow::Result;
use moni_lib::meta::Meta;
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

/// GLOBAL_REQUEST_COUNT is a global request count
/// static GLOBAL_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1);

/// start server
pub async fn start(addr: SocketAddr, meta: &Meta) -> Result<()> {
    let output_path = meta.get_output();
    // init global wasm worker pool
    let pool = moni_runtime::create_pool(&output_path)?;
    // set to runtime wasm pool
    moni_runtime::server::WASM_INSTANCES.insert(output_path.clone(), Arc::new(pool));
    info!("wasm pool created");

    // set output as default
    moni_runtime::server::DEFAULT_WASM_PATH
        .set(output_path)
        .unwrap();

    // start http server in runtime crates
    moni_runtime::server::start(addr).await?;
    Ok(())
}
