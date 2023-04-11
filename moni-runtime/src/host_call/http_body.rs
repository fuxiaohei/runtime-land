use self::http_body::{BodyError, HttpBodyHandle};
use super::HttpContext;
use futures_util::StreamExt;
use hyper::body::Body;

wasmtime::component::bindgen!({
    world:"http-body",
    path: "../wit",
    async: true,
});

#[async_trait::async_trait]
impl http_body::Host for HttpContext {
    async fn http_body_read(
        &mut self,
        handle: HttpBodyHandle,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError {}));
        }
        let body = self.body_map.get_mut(&handle).unwrap();
        let chunk = body.next().await;
        if chunk.is_none() {
            return Ok(Ok((vec![], true))); // end of stream
        }
        let chunk = chunk.unwrap().unwrap();
        println!("----read chunk: {:?}", chunk.len());
        Ok(Ok((chunk.to_vec(), false)))
    }

    async fn http_body_read_all(
        &mut self,
        handle: HttpBodyHandle,
    ) -> wasmtime::Result<Result<Vec<u8>, BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError {}));
        }
        let body = self.body_map.get_mut(&handle).unwrap();
        let data = hyper::body::to_bytes(body)
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|e| {
                println!("----read all error: {:?}", e);
                BodyError {}
            });
        Ok(data)
    }

    async fn http_body_write(
        &mut self,
        handle: HttpBodyHandle,
        data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, BodyError>> {
        if !self.body_map.contains_key(&handle) {
            return Ok(Err(BodyError {}));
        }
        let size = data.len() as u64;
        let sender = self.body_sender_map.get_mut(&handle).unwrap();
        sender
            .send_data(data.into())
            .await
            .map_err(|_| BodyError {})?;
        Ok(Ok(size))
    }

    async fn http_body_new(&mut self) -> wasmtime::Result<Result<HttpBodyHandle, BodyError>> {
        let (sender, body) = Body::channel();
        let body_handle = self.set_body(body);
        self.set_body_sender(body_handle, sender);
        Ok(Ok(body_handle))
    }
}
