use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};
use wit_component::ComponentEncoder;

mod guest;
mod imports;

pub struct Context {
    wasi_ctx: WasiCtx,
    table: Table,
    http_host: imports::HttpServiceHostImpl,
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

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        let mut table = Table::new();
        Context {
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stdio()
                .build(&mut table)
                .unwrap(),
            table,
            http_host: imports::HttpServiceHostImpl {},
        }
    }

    /// get http_ctx
    pub fn http_host(&mut self) -> &mut imports::HttpServiceHostImpl {
        &mut self.http_host
    }
}

fn encode_wasm_component(path: &str, output: Option<String>) {
    let file_bytes = std::fs::read(path).expect("parse wasm file error");
    let wasi_adapter =
        std::fs::read("./crates/runtime/engine/wasi_snapshot_preview1.reactor.wasm").unwrap();

    let component = ComponentEncoder::default()
        .module(&file_bytes)
        .expect("Pull custom sections from module")
        .validate(true)
        .adapter("wasi_snapshot_preview1", &wasi_adapter)
        .expect("Add adapter to component")
        .encode()
        .expect("Encode component");

    let output = output.unwrap_or_else(|| path.to_string());
    std::fs::write(&output, component).expect("Write component file error");
    println!("Convert wasm module to component success, {}", &output)
}

fn create_wasmtime_config() -> Config {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    call_wasm().await;
}

async fn call_wasm() {
    let target = "target/wasm32-wasi/release/wit_v2_guest.wasm";
    let output = "target/wasm32-wasi/release/wit_v2_guest.component.wasm";

    encode_wasm_component(target, Some(output.to_string()));
    println!("Run component: {}", output);

    let engine = Engine::new(&create_wasmtime_config()).unwrap();
    let component = Component::from_file(&engine, output).unwrap();
    let mut linker: Linker<Context> = Linker::new(&engine);

    // init wasi context
    wasmtime_wasi::preview2::wasi::command::add_to_linker(&mut linker)
        .expect("add wasi::command linker failed");
    imports::HttpService::add_to_linker(&mut linker, Context::http_host)
        .expect("add http service failed");

    // init context
    let context = Context::new();
    let mut store = Store::new(&engine, context);

    // get export function
    let (exports, _) =
        crate::guest::HttpHandler::instantiate_async(&mut store, &component, &linker)
            .await
            .unwrap();
    let req_arg = guest::exports::land::http::http_incoming::Request {
        method: "GET",
        uri: "/abc",
        headers: &vec![(String::from("x-a"), String::from("b"))],
        body: None,
    };
    let resp = exports
        .land_http_http_incoming()
        .call_handle_request(&mut store, req_arg)
        .await
        .unwrap();
    println!("resp: {:?}", resp);
}
