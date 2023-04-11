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
        Ok(Err(BodyError {}))
    }

    async fn http_body_new(&mut self) -> wasmtime::Result<Result<HttpBodyHandle, BodyError>> {
        let body = Body::empty();
        let body_handle = self.set_body(body);
        Ok(Ok(body_handle))
    }
}
