use anyhow::Result;
use lol_core::meta::Meta;
use std::{net::SocketAddr, sync::Arc};
use tracing::debug;

/// GLOBAL_REQUEST_COUNT is a global request count
/// static GLOBAL_REQUEST_COUNT: AtomicU64 = AtomicU64::new(1);

/// start server
pub async fn start(addr: SocketAddr, meta: &Meta) -> Result<()> {
    let output_path = meta.get_output();
    // init global wasm worker pool
    let pool = lol_runtime::create_pool(&output_path)?;
    // set to runtime wasm pool
    lol_runtime::server::WASM_INSTANCES.insert(output_path.clone(), Arc::new(pool));
    debug!(output_path = output_path, "wasm pool created");

    // set output as default
    lol_runtime::server::DEFAULT_WASM_PATH
        .set(output_path.clone())
        .unwrap();
    debug!(output_path = output_path, "wasm pool set to default");

    // start http server in runtime crates
    lol_runtime::server::start(addr).await?;
    Ok(())
}
