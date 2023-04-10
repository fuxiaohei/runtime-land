use crate::host_call::{http_incoming, HttpImplContext, HttpInterface};
use anyhow::Result;
use axum::body::Body;
use std::fmt::Debug;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_host::WasiCtx;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine, Store};

pub struct Context {
    wasi_ctx: WasiCtx,
    http_impl_ctx: HttpImplContext,
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
            http_impl_ctx: HttpImplContext::new(1),
        }
    }

    /// get wasi
    pub fn wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    /// get http_impl_ctx
    pub fn http_impl_ctx(&mut self) -> &mut HttpImplContext {
        &mut self.http_impl_ctx
    }

    /// set body
    pub fn set_body(&mut self, body: Body) -> u32 {
        self.http_impl_ctx.set_body(body)
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
        wasi_host::command::add_to_linker(&mut linker, Context::wasi)?;
        HttpInterface::add_to_linker(&mut linker, Context::http_impl_ctx)?;

        Ok(Self {
            path: path.to_string(),
            engine,
            instance_pre: linker.instantiate_pre(&component)?,
        })
    }

    /// handle_request is used to handle http request
    pub async fn handle_request(
        &mut self,
        req: http_incoming::RequestParam<'_>,
        context: Context,
    ) -> Result<http_incoming::Response> {
        // create store
        let mut store = Store::new(&self.engine, context);

        // get exports and call handle_request
        let (exports, _instance) =
            HttpInterface::instantiate_pre(&mut store, &self.instance_pre).await?;
        let resp = exports
            .http_incoming()
            .call_handle_request(&mut store, req)
            .await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        host_call::http_incoming::RequestParam,
        worker::{Context, Worker},
    };

    #[tokio::test]
    async fn run_wasm() {
        let wasm_file = "../tests/data/rust_impl.component.wasm";
        let mut worker = Worker::new(wasm_file).await.unwrap();

        for _ in 1..10 {
            let headers: Vec<(&str, &str)> = vec![];
            let req = RequestParam {
                method: "GET",
                uri: "/abc",
                headers: &headers,
                body: Some(2),
            };

            let resp = worker.handle_request(req, Context::new()).await.unwrap();
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