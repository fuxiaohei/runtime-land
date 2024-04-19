use super::host::land::http::body::{BodyError, BodyHandle, Host};
use super::HttpContext;

#[async_trait::async_trait]
impl Host for HttpContext {
    async fn read(
        &mut self,
        handle: BodyHandle,
        size: u32,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), BodyError>> {
        Ok(self.read_body(handle, size).await)
    }

    async fn read_all(
        &mut self,
        handle: BodyHandle,
    ) -> wasmtime::Result<Result<Vec<u8>, BodyError>> {
        Ok(self.read_body_all(handle).await)
    }

    async fn write(
        &mut self,
        handle: BodyHandle,
        data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, BodyError>> {
        Ok(self.write_body(handle, data).await)
    }

    async fn new(&mut self) -> wasmtime::Result<Result<BodyHandle, BodyError>> {
        return Ok(Ok(self.new_empty_body()));
    }

    async fn new_stream(&mut self) -> wasmtime::Result<Result<BodyHandle, BodyError>> {
        return Ok(Ok(self.new_writable_stream()));
    }
}
