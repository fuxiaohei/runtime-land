wasmtime::component::bindgen!({
    world: "http-service",
    path: "./wit",
    async:true,
});

impl land::http::http_outgoing::RequestOptions {
    pub fn key(&self) -> String {
        format!("t-{}-r-{:?}", self.timeout, self.redirect)
    }
}
