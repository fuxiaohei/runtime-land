use crate::host_call::{HttpContext, HttpHandler, HttpService, Request, Response};
use anyhow::Result;
use hyper::body::Body;
use std::fmt::Debug;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig, Store};
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};

pub struct Context {
    wasi_ctx: WasiCtx,
    table: Table,
    http_ctx: HttpContext,
}

impl Default for Context {
    fn default() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string())
    }
}

impl WasiView for Context {
    fn table(&self) -> &Table {
        &self.table
    }
    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
    fn ctx(&self) -> &WasiCtx {
        &self.wasi_ctx
    }
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl Context {
    pub fn new(req_id: String) -> Self {
        let mut table = Table::new();
        Context {
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stdio()
                .build(&mut table)
                .unwrap(),
            http_ctx: HttpContext::new(req_id),
            table,
        }
    }

    /// get http_ctx
    pub fn http_ctx(&mut self) -> &mut HttpContext {
        &mut self.http_ctx
    }

    /// set body
    pub fn set_body(&mut self, body: Body) -> u32 {
        self.http_ctx.set_body(body)
    }

    /// take body
    pub fn take_body(&mut self, handle: u32) -> Option<Body> {
        self.http_ctx.take_body(handle)
    }

    /// get request id
    pub fn req_id(&self) -> String {
        self.http_ctx.req_id.clone()
    }
}

fn create_wasmtime_config() -> Config {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);

    const MB: usize = 1 << 20;
    let mut pooling_allocation_config = PoolingAllocationConfig::default();
    pooling_allocation_config.instance_size(MB);
    pooling_allocation_config.instance_memory_pages(128 * (MB as u64) / (64 * 1024));
    config.allocation_strategy(InstanceAllocationStrategy::Pooling(
        pooling_allocation_config,
    ));

    config
}

/// Worker is used to run wasm component
pub struct Worker {
    path: String,
    engine: Engine,
    // component: Component,
    instance_pre: InstancePre<Context>,
}

impl Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker").field("path", &self.path).finish()
    }
}

impl Worker {
    /// new a worker
    pub async fn new(path: &str) -> Result<Self> {
        let binary = std::fs::read(path)?;
        Self::from_binary(&binary).await
    }

    // from_binary is used to create worker from bytes
    pub async fn from_binary(bytes: &[u8]) -> Result<Self> {
        // create component
        let config = create_wasmtime_config();
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, bytes)?;

        // create linker
        let mut linker: Linker<Context> = Linker::new(&engine);
        // init wasi context
        wasmtime_wasi::preview2::wasi::command::add_to_linker(&mut linker)
            .expect("add wasmtime_wasi::preview2 failed");
        HttpService::add_to_linker(&mut linker, Context::http_ctx)?;

        Ok(Self {
            path: "bytes".to_string(),
            engine,
            instance_pre: linker.instantiate_pre(&component)?,
        })
    }

    /// handle_request is used to handle http request
    pub async fn handle_request(
        &mut self,
        req: Request<'_>,
        context: Context,
    ) -> Result<(Response, Body)> {
        // create store
        let mut store = Store::new(&self.engine, context);

        // get exports and call handle_request
        let (exports, _instance) =
            HttpHandler::instantiate_pre(&mut store, &self.instance_pre).await?;
        let resp = exports
            .land_http_http_incoming()
            .call_handle_request(&mut store, req)
            .await?;
        let body = store.data_mut().take_body(resp.body.unwrap()).unwrap();
        Ok((resp, body))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        host_call::Request,
        worker::{Context, Worker},
    };
    use hyper::Body;

    #[tokio::test]
    async fn run_wasm() {
        let wasm_file = "../../tests/data/rust_impl.component.wasm";
        let mut worker = Worker::new(wasm_file).await.unwrap();

        for _ in 1..10 {
            let headers: Vec<(String, String)> = vec![];

            let mut context = Context::default();
            let body = Body::from("test request body");
            let body_handle = context.set_body(body);

            let req = Request {
                method: "GET",
                uri: "/abc",
                headers: &headers,
                body: Some(body_handle),
            };

            let (resp, _body) = worker.handle_request(req, context).await.unwrap();
            assert_eq!(resp.status, 200);
            // this wasm return request's body
            // so the body handler u32 is 2, same as request's body
            assert_eq!(resp.body, Some(2));

            let headers = resp.headers;
            for (key, value) in headers {
                if key == "X-Request-Method" {
                    assert_eq!(value, "GET");
                }
                if key == "X-Request-Url" {
                    assert_eq!(value, "/abc");
                }
            }
        }
    }
}
