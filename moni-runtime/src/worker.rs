use std::fmt::Debug;

use crate::host_call::fetch_impl::{http_fetch, FetchCtx};
use crate::host_call::http_impl;
use anyhow::Result;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_host::WasiCtx;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine, Store};

pub struct Context {
    wasi_ctx: WasiCtx,
    fetch_ctx: FetchCtx,
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
            fetch_ctx: FetchCtx::new(1),
        }
    }
    /// get wasi
    pub fn wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    /// get fetch ctx
    pub fn fetch(&mut self) -> &mut FetchCtx {
        &mut self.fetch_ctx
    }
}

fn create_wasmtime_config() -> Config {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
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
        wasi_host::add_to_linker(&mut linker, Context::wasi)?;
        http_fetch::add_to_linker(&mut linker, Context::fetch)?;

        Ok(Self {
            path: path.to_string(),
            engine,
            instance_pre: linker.instantiate_pre(&component)?,
        })
    }

    /// handle_request is used to handle http request
    pub async fn handle_request(
        &mut self,
        req: http_impl::http_handler::Request<'_>,
    ) -> Result<http_impl::http_handler::Response> {
        // create store
        let mut store = Store::new(&self.engine, Context::new());

        // get exports and call handle_request
        let (exports, _instance) =
            http_impl::HttpHandler::instantiate_pre(&mut store, &self.instance_pre).await?;
        let resp = exports
            .http_handler()
            .call_handle_request(&mut store, req)
            .await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::Worker;
    use crate::host_call::http_impl::http_handler::Request;

    #[tokio::test]
    async fn run_wasm() {
        let wasm_file = "../tests/data/rust_basic.component.wasm";
        let mut worker = Worker::new(wasm_file).await.unwrap();

        for _ in 1..10 {
            let headers: Vec<(&str, &str)> = vec![];
            let req = Request {
                method: "GET",
                uri: "/abc",
                headers: &headers,
                body: Some("xxxyyy".as_bytes()),
            };

            let resp = worker.handle_request(req).await.unwrap();
            assert_eq!(resp.status, 200);
            assert_eq!(resp.body, Some("Hello, World".as_bytes().to_vec()));

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
