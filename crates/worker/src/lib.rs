use anyhow::Result;
use context::Context;
use hyper::body::Incoming;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig, Store};

mod context;
mod hostcall;

fn create_config() -> Config {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);

    // SIMD support requires SSE3 and SSSE3 on x86_64.
    // in docker container, it will cause error
    config.wasm_simd(false);

    // const MB: usize = 1 << 20;
    // let mut pooling_allocation_config = PoolingAllocationConfig::default();
    // pooling_allocation_config.max_core_instance_size(MB);
    // pooling_allocation_config.max_memories_per_component(128 * (MB as u32) / (64 * 1024));
    let pooling_allocation_config = PoolingAllocationConfig::default();
    config.allocation_strategy(InstanceAllocationStrategy::Pooling(
        pooling_allocation_config,
    ));

    config
}

/// Worker is used to run wasm component
#[derive(Clone)]
pub struct Worker {
    path: String,
    engine: Engine,
    instance_pre: InstancePre<Context>,
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker").field("path", &self.path).finish()
    }
}

impl Worker {
    // from_binary is used to create worker from bytes
    pub async fn from_binary(bytes: &[u8], path: Option<String>) -> Result<Self> {
        // create component
        let config = create_config();
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, bytes)?;

        // create linker
        let mut linker: Linker<Context> = Linker::new(&engine);
        // init wasi context
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)
            .expect("add wasmtime_wasi::preview2 failed");
        hostcall::HttpService::add_to_linker(&mut linker, Context::http_ctx)
            .expect("add http_service failed");

        Ok(Self {
            path: path.unwrap_or("binary".to_string()),
            engine,
            instance_pre: linker.instantiate_pre(&component)?,
        })
    }

    /// new a worker from path
    pub async fn new(path: &str) -> Result<Self> {
        let binary = std::fs::read(path)?;
        Self::from_binary(&binary, Some(path.to_string())).await
    }

    /// handle_request is used to handle http request
    pub async fn handle_request(
        &self,
        req: hostcall::Request,
        context: Context,
    ) -> Result<(hostcall::Response, Incoming)> {
        // create store
        let mut store = Store::new(&self.engine, context);

        // get exports and call handle_request
        let (exports, _instance) =
            hostcall::HttpHandler::instantiate_pre(&mut store, &self.instance_pre).await?;
        let resp = exports
            .land_http_incoming()
            .call_handle_request(&mut store, &req)
            .await?;
        let body = store.data_mut().take_body(resp.body.unwrap()).unwrap();
        Ok((resp, body))
    }
}
