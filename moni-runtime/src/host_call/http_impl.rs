wasmtime::component::bindgen!({
    world:"http-handler",
    path: "../wit/http-handler.wit",
    async:true,
});
