use crate::engine::MODULE_VERSION;
use crate::hostcall;
use anyhow::Result;
use axum::body::Body;
use tracing::debug;
use wasmtime::UpdateDeadline;
use wasmtime::{
    component::{Component, InstancePre, Linker},
    Engine, Store,
};

/// Worker is used to run wasm component
#[derive(Clone)]
pub struct Worker {
    path: String,
    engine: Engine,
    instance_pre: InstancePre<super::Context>,
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker").field("path", &self.path).finish()
    }
}

impl Worker {
    // from_binary is used to create worker from bytes
    pub async fn from_binary(bytes: &[u8], path: Option<String>) -> Result<Self> {
        let engine = super::engine::get("default")?;
        let component = Component::from_binary(&engine, bytes)?;
        debug!("Load wasm component from binary, size:{}", bytes.len());

        // create linker
        let mut linker: Linker<super::Context> = Linker::new(&engine);
        // init wasi context
        wasmtime_wasi::command::add_to_linker(&mut linker)
            .expect("add wasmtime_wasi::preview2 failed");
        hostcall::HttpService::add_to_linker(&mut linker, super::Context::http_ctx)
            .expect("add http_service failed");

        Ok(Self {
            path: path.unwrap_or("binary".to_string()),
            engine,
            instance_pre: linker.instantiate_pre(&component)?,
        })
    }

    async fn from_aot(path: String) -> Result<Self> {
        let engine = super::engine::get("default")?;
        let bytes = std::fs::read(&path)?;
        debug!(
            "Load wasm component from AOT file: {}, size: {}",
            path,
            bytes.len()
        );

        let component = unsafe { Component::deserialize(&engine, bytes)? };

        // create linker
        let mut linker: Linker<super::Context> = Linker::new(&engine);
        // init wasi context
        wasmtime_wasi::command::add_to_linker(&mut linker)
            .expect("add wasmtime_wasi::preview2 failed");
        hostcall::HttpService::add_to_linker(&mut linker, super::Context::http_ctx)
            .expect("add http_service failed");

        Ok(Self {
            path,
            engine,
            instance_pre: linker.instantiate_pre(&component)?,
        })
    }

    pub fn compile_aot(src: &str, dst: &str) -> Result<()> {
        let engine = super::engine::get("default")?;
        let component = Component::from_file(&engine, src)?;
        let bytes = Component::serialize(&component)?;
        debug!("Write AOT from {} to {}, size: {}", src, dst, bytes.len());
        std::fs::write(dst, bytes)?;
        Ok(())
    }

    /// new a worker from path
    pub async fn new(path: &str, is_aot: bool) -> Result<Self> {
        let binary = std::fs::read(path)?;

        // compile aot wasm
        if is_aot {
            let suffix = format!(".wasm.{}.aot", MODULE_VERSION);
            let aot_path = path.replace(".wasm", &suffix);
            if std::path::Path::new(&aot_path).exists() {
                return Self::from_aot(aot_path).await;
            }
            let path2 = path.to_string();
            std::thread::spawn(move || {
                match Self::compile_aot(&path2, &aot_path) {
                    Ok(_) => debug!("Compile AOT success: {}", &aot_path),
                    Err(e) => debug!("Compile AOT failed: {}", e),
                };
            });
        }
        Self::from_binary(&binary, Some(path.to_string())).await
    }

    /// handle_request is used to handle http request
    pub async fn handle_request(
        &self,
        req: hostcall::Request,
        context: super::Context,
    ) -> Result<(hostcall::Response, Body)> {
        // create store
        let mut store = Store::new(&self.engine, context);
        store.set_epoch_deadline(1);
        store.epoch_deadline_callback(move |store| {
            debug!(
                "epoch_deadline_callback, cost:{:.2?}",
                store.data().elapsed()
            );
            Ok(UpdateDeadline::Yield(1))
        });
        store.limiter(|ctx| &mut ctx.limiter);

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
