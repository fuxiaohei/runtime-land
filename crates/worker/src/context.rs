use super::hostcall::HttpContext;
use axum_core::body::Body;
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};

pub struct Context {
    wasi_ctx: WasiCtx,
    table: Table,
    http_ctx: HttpContext,
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
        let table = Table::new();
        Context {
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stderr()
                .inherit_stdout()
                .build(),
            http_ctx: HttpContext::new(req_id),
            table,
        }
    }

    /// get http_ctx
    pub fn http_ctx(&mut self) -> &mut HttpContext {
        &mut self.http_ctx
    }

    /// get request id
    pub fn req_id(&self) -> String {
        self.http_ctx.req_id.clone()
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
