use self::http_body::{BodyError, HttpBodyHandle};
use super::HttpContext;
use futures_util::StreamExt;
use hyper::body::Body;
use tracing::{debug, instrument};

wasmtime::component::bindgen!({
    world:"http-body",
    path: "../wit",
    async: true,
});

#[async_trait::async_trait]
impl http_body::Host for HttpContext {
    #[instrument(skip_all, name = "[Body]", level = "debug", fields(req_id = self.req_id))]
    async fn http_body_read(
        &mut self,
        handle: HttpBodyHandle,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError::InvalidHandle));
        }
        let body = self.body_map.get_mut(&handle).unwrap();
        let chunk = body.next().await;
        if chunk.is_none() {
            return Ok(Ok((vec![], true))); // end of stream
        }
        let chunk = chunk.unwrap().unwrap();
        debug!("read chunk: {}", chunk.len());
        Ok(Ok((chunk.to_vec(), false)))
    }

    #[instrument(skip_all, name = "[Body]", level = "debug", fields(req_id = self.req_id))]
    async fn http_body_read_all(
        &mut self,
        handle: HttpBodyHandle,
    ) -> wasmtime::Result<Result<Vec<u8>, BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError::InvalidHandle));
        }
        let body = self.body_map.get_mut(&handle).unwrap();
        let data = hyper::body::to_bytes(body)
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|e| BodyError::ReadFailed(e.to_string()));
        debug!("read all: {}", data.as_ref().unwrap().len());
        Ok(data)
    }

    #[instrument(skip_all, name = "[Body]", level = "debug", fields(req_id = self.req_id))]
    async fn http_body_write(
        &mut self,
        handle: HttpBodyHandle,
        data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError::InvalidHandle));
        }
        let size = data.len() as u64;
        if !self.body_sender_map.contains_key(&handle) {
            let body = Body::from(data);
            self.replace_body(handle, body);
            debug!("write body: {}", size);
            return Ok(Ok(size));
        }
        let sender = self.body_sender_map.get_mut(&handle).unwrap();
        sender
            .send_data(data.into())
            .await
            .map_err(|e| BodyError::WriteFailed(e.to_string()))?;
        debug!("write chunk: {}", size);
        Ok(Ok(size))
    }

    #[instrument(skip_all, name = "[Body]", level = "debug", fields(req_id = self.req_id))]
    async fn http_body_new(&mut self) -> wasmtime::Result<Result<HttpBodyHandle, BodyError>> {
        let body = Body::empty();
        let body_handle = self.set_body(body);
        debug!("new body: {}", body_handle);
        Ok(Ok(body_handle))
    }

    #[instrument(skip_all, name = "[Body]", level = "debug", fields(req_id = self.req_id))]
    async fn http_body_new_stream(
        &mut self,
    ) -> wasmtime::Result<Result<HttpBodyHandle, BodyError>> {
        let (sender, body) = Body::channel();
        let body_handle = self.set_body(body);
        self.set_body_sender(body_handle, sender);
        debug!("new body stream: {}", body_handle);
        Ok(Ok(body_handle))
    }
}
