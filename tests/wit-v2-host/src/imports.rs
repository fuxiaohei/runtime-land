wasmtime::component::bindgen!({
    world: "http-service",
    path: "../../wit",
    async:true,
});

use land::http::{http_body, http_outgoing, http_types};
pub struct HttpServiceHostImpl {}

impl http_types::Host for HttpServiceHostImpl {}

#[async_trait::async_trait]
impl http_body::Host for HttpServiceHostImpl {
    async fn http_body_read(
        &mut self,
        _handle: http_body::HttpBodyHandle,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), http_body::BodyError>> {
        Ok(Err(http_body::BodyError::InvalidHandle))
    }

    async fn http_body_read_all(
        &mut self,
        _handle: http_body::HttpBodyHandle,
    ) -> wasmtime::Result<Result<Vec<u8>, http_body::BodyError>> {
        Ok(Err(http_body::BodyError::InvalidHandle))
    }

    async fn http_body_write(
        &mut self,
        _handle: http_body::HttpBodyHandle,
        _data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, http_body::BodyError>> {
        Ok(Err(http_body::BodyError::InvalidHandle))
    }

    async fn http_body_new(
        &mut self,
    ) -> wasmtime::Result<Result<http_body::HttpBodyHandle, http_body::BodyError>> {
        Ok(Err(http_body::BodyError::InvalidHandle))
    }

    async fn http_body_new_stream(
        &mut self,
    ) -> wasmtime::Result<Result<http_body::HttpBodyHandle, http_body::BodyError>> {
        Ok(Err(http_body::BodyError::InvalidHandle))
    }
}

#[async_trait::async_trait]
impl http_outgoing::Host for HttpServiceHostImpl {
    async fn fetch_request(
        &mut self,
        _req: http_outgoing::Request,
        _options: http_outgoing::RequestOptions,
    ) -> wasmtime::Result<Result<http_outgoing::Response, http_outgoing::RequestError>> {
        Ok(Err(http_outgoing::RequestError::NetworkError))
    }
}
