use crate::hostcall::HttpContext;
use axum::body::Body;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::preview2::{WasiCtx, WasiCtxBuilder, WasiView};

pub struct Context {
    wasi_ctx: WasiCtx,
    table: ResourceTable,
    http_ctx: HttpContext,
}

impl WasiView for Context {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        let table = ResourceTable::new();
        Context {
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stderr()
                .inherit_stdout()
                .build(),
            http_ctx: HttpContext::new(),
            table,
        }
    }
    /// get http_ctx
    pub fn http_ctx(&mut self) -> &mut HttpContext {
        &mut self.http_ctx
    }
    /// take body
    pub fn take_body(&mut self, handle: u32) -> Option<Body> {
        self.http_ctx.take_body(handle)
    }
    /// set body
    pub fn set_body(&mut self, handle: u32, body: Body) -> u32 {
        self.http_ctx.set_body(handle, body)
    }
}
