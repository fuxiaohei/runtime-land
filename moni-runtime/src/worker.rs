use crate::host_call::http_body::http_body;
use crate::host_call::{http_incoming, http_incoming::HttpIncoming, HttpContext};
use crate::worker::http_incoming::http_incoming::{Request, Response};
use anyhow::Result;
use hyper::body::Body;
use std::fmt::Debug;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_host::WasiCtx;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig, Store};

pub struct Context {
    wasi_ctx: WasiCtx,
    http_ctx: HttpContext,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            wasi_ctx: WasiCtxBuilder::new().inherit_stdio().build(),
            http_ctx: HttpContext::new(1),
        }
    }

    /// get wasi
    pub fn wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
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
        // create component
        let config = create_wasmtime_config();
        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, path)?;

        // create linker
        let mut linker: Linker<Context> = Linker::new(&engine);
        wasi_host::command::add_to_linker(&mut linker, Context::wasi)?;
        http_body::add_to_linker(&mut linker, Context::http_ctx)?;

        Ok(Self {
            path: path.to_string(),
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
            HttpIncoming::instantiate_pre(&mut store, &self.instance_pre).await?;
        let resp = exports
            .http_incoming()
            .call_handle_request(&mut store, req)
            .await?;
        let body = store.data_mut().take_body(resp.body.unwrap()).unwrap();
        Ok((resp, body))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        host_call::http_incoming::http_incoming::Request,
        worker::{Context, Worker},
    };
    use hyper::Body;

    #[tokio::test]
    async fn run_wasm() {
        let wasm_file = "../tests/data/rust_impl.component.wasm";
        let mut worker = Worker::new(wasm_file).await.unwrap();

        for _ in 1..10 {
            let headers: Vec<(&str, &str)> = vec![];

            let mut context = Context::new();
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
