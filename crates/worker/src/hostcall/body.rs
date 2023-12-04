use super::host::land::http::body::{BodyError, BodyHandle, Host};
use super::HttpContext;

#[async_trait::async_trait]
impl Host for HttpContext {
    async fn read(
        &mut self,
        _handle: BodyHandle,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }
    async fn read_all(
        &mut self,
        _handle: BodyHandle,
    ) -> wasmtime::Result<Result<Vec<u8>, BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }
    async fn write(
        &mut self,
        _handle: BodyHandle,
        _data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }
    async fn new_static(&mut self) -> wasmtime::Result<Result<BodyHandle, BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }
    async fn new_stream(&mut self) -> wasmtime::Result<Result<BodyHandle, BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }
}
