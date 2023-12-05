use super::host::land::http::body::{BodyError, BodyHandle, Host};
use super::HttpContext;
use axum_core::body::Body;
use tracing::debug;

#[async_trait::async_trait]
impl Host for HttpContext {
    async fn read(
        &mut self,
        handle: BodyHandle,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError::InvalidHandle));
        }
        let chunk = self.body_map.get_mut(&handle).unwrap().read().await;
        if chunk.is_none() {
            return Ok(Ok((vec![], true))); // end of stream
        }
        let chunk = chunk.unwrap();
        debug!("read chunk: {}, handle: {}", chunk.len(), handle);
        Ok(Ok((chunk.to_vec(), false)))
    }

    async fn read_all(
        &mut self,
        handle: BodyHandle,
    ) -> wasmtime::Result<Result<Vec<u8>, BodyError>> {
        // if body is in body_map, read all bytes
        if self.body_map.contains_key(&handle) {
            let body = self.body_map.get_mut(&handle).unwrap();
            let bytes = body.read_all().await.unwrap();
            return Ok(Ok(bytes.to_vec()));
        }
        return Ok(Err(BodyError::InvalidHandle));
    }

    async fn write(
        &mut self,
        handle: BodyHandle,
        data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }

    async fn new_static(&mut self) -> wasmtime::Result<Result<BodyHandle, BodyError>> {
        let empty = Body::empty();
        let handle = self.set_outgoing_body(empty);
        return Ok(Ok(handle));
    }

    async fn new_stream(&mut self) -> wasmtime::Result<Result<BodyHandle, BodyError>> {
        return Ok(Err(BodyError::InvalidHandle));
    }
}
