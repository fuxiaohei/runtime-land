use self::host::land::http::body::BodyError;
use axum_core::body::{Body, BodyDataStream};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

mod body;
mod guest;
mod host;
mod outgoing;

pub use guest::exports::land::http::incoming::{Request, Response};
pub use guest::HttpHandler;
pub use host::HttpService;

/*
type HttpContextBody = (Body, Option<BodyError>);
type HttpCoontextStream = (BodyDataStream, Option<BodyError>);
*/

pub struct HttpContext {
    /// req_id is the unique request id for each request.
    pub req_id: String,
    /// fetch_counter is the counter for fetch.
    pub fetch_counter: u16,

    /// body_map is the map for body.
    body_map: HashMap<u32, Body>,
    /// body_stream_map is the map for body stream.
    body_stream_map: HashMap<u32, BodyDataStream>,
    /// body_read_temp is the temp buffer for body read.
    body_read_temp: HashMap<u32, Vec<u8>>,
    /// body_read_end_map is the map for body read end.
    body_read_end_map: HashMap<u32, bool>,
    /// body_id is incremented for each body.
    body_id: AtomicU32,
}

impl HttpContext {
    pub fn new(req_id: String) -> Self {
        Self {
            req_id,
            fetch_counter: 10,
            body_map: HashMap::new(),
            body_stream_map: HashMap::new(),
            body_read_temp: HashMap::new(),
            body_read_end_map: HashMap::new(),
            body_id: AtomicU32::new(1),
        }
    }

    pub fn take_body(&mut self, handle: u32) -> Option<Body> {
        return self.body_map.remove(&handle);
    }

    pub fn set_body(&mut self, mut handle: u32, body: Body) -> u32 {
        if handle < 1 {
            handle = self
                .body_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        self.body_map.insert(handle, body);
        handle
    }

    pub async fn read_body(
        &mut self,
        handle: u32,
        size: u32,
    ) -> Result<(Vec<u8>, bool), BodyError> {
        if self.body_read_end_map.contains_key(&handle) {
            // if read end, return empty chunk and true flag
            return Ok((vec![], true));
        }

        if !self.body_stream_map.contains_key(&handle) {
            // convert body to stream and read chunk
            if self.body_map.contains_key(&handle) {
                let body = self.body_map.remove(&handle).unwrap();
                let stream = body.into_data_stream();
                self.body_stream_map.insert(handle, stream);
            } else {
                return Err(BodyError::InvalidHandle);
            }
        }

        // if prev chunk is over size limit, split it
        let prev_chunk = self.body_read_temp.remove(&handle).unwrap_or_default();
        if prev_chunk.len() as u32 > size {
            let (first, second) = prev_chunk.split_at(size as usize);
            self.body_read_temp.insert(handle, second.to_vec());
            return Ok((first.to_vec(), false));
        }

        let stream = self.body_stream_map.get_mut(&handle).unwrap();
        let chunk = stream.next().await;

        if chunk.is_none() {
            // no new chunk, return prev chunk if exist
            if !prev_chunk.is_empty() {
                return Ok((prev_chunk, false));
            }
            // read end
            return Ok((vec![], true));
        }

        let chunk = chunk.unwrap();
        if chunk.is_err() {
            // read error
            return Err(BodyError::ReadFailed(chunk.err().unwrap().to_string()));
        }
        let mut chunk = chunk.unwrap();
        if !prev_chunk.is_empty() {
            let prev_bytes = Bytes::from(prev_chunk);
            chunk = [prev_bytes, chunk].concat().into();
        }
        if chunk.len() as u32 > size {
            // we need to split chunk, cache the chunk in some where
            // and return the first size bytes
            // and return the rest bytes in next read
            let (first, second) = chunk.split_at(size as usize);
            self.body_read_temp.insert(handle, second.to_vec());
            return Ok((first.to_vec(), false));
        }
        // return the whole chunk. if chunk exist, flag must be false
        // make sure that when flag is true, chunk is empty
        Ok((chunk.to_vec(), false))
    }

    pub async fn read_body_all(&mut self, handle: u32) -> Result<Vec<u8>, BodyError> {
        // if handle in body_read_end_map, return read-ended error
        if self.body_read_end_map.contains_key(&handle) {
            return Err(BodyError::ReadEnded);
        }

        // if handle in body_map, use axum_core::body::to_bytes
        if self.body_map.contains_key(&handle) {
            let body = self.body_map.remove(&handle).unwrap();
            let bytes = axum::body::to_bytes(body, usize::MAX)
                .await
                .map_err(|err| BodyError::ReadFailed(err.to_string()))?;
            self.body_read_end_map.insert(handle, true);
            return Ok(bytes.to_vec());
        }
        // if handle in body_stream_map, use body_stream_map to read all
        if self.body_stream_map.contains_key(&handle) {
            let prev_chunk = self.body_read_temp.remove(&handle).unwrap_or_default();
            let stream = self.body_stream_map.remove(&handle).unwrap();
            let bytes = stream
                .try_collect::<Vec<_>>()
                .await
                .map_err(|err| BodyError::ReadFailed(err.to_string()))?;
            self.body_read_end_map.insert(handle, true);
            if !prev_chunk.is_empty() {
                return Ok([prev_chunk, bytes.concat()].concat().to_vec());
            }
            return Ok(bytes.concat().to_vec());
        }

        // if handle not exist, return error
        return Err(BodyError::InvalidHandle);
    }
}

impl host::land::http::types::Host for HttpContext {}

#[cfg(test)]
mod tests {
    use super::HttpContext;
    use axum_core::body::Body;

    #[tokio::test]
    async fn test_http_context_body_read() {
        let mut context = HttpContext::new("test-id".to_string());
        let body = Body::from("abc".repeat(10));
        let body_handle = context.set_body(0, body);
        assert_eq!(body_handle, 1);

        // read 5 bytes
        let (chunk, flag) = context.read_body(body_handle, 5).await.unwrap();
        assert_eq!(chunk.len(), 5);
        assert_eq!(String::from_utf8(chunk).unwrap(), "abcab");
        assert_eq!(flag, false);

        // read next 5 bytes
        let (chunk, flag) = context.read_body(body_handle, 5).await.unwrap();
        assert_eq!(chunk.len(), 5);
        assert_eq!(String::from_utf8(chunk).unwrap(), "cabca");
        assert_eq!(flag, false);

        // read over left size
        let (chunk, flag) = context.read_body(body_handle, 100).await.unwrap();
        assert_eq!(chunk.len(), 20);
        assert_eq!(flag, false);

        // read end
        let (chunk, flag) = context.read_body(body_handle, 100).await.unwrap();
        assert_eq!(chunk.len(), 0);
        assert_eq!(flag, true);
    }

    #[tokio::test]
    async fn test_http_context_body_read_all() {
        let mut context = HttpContext::new("test-id".to_string());
        let body = Body::from("abc".repeat(10));
        let body_handle = context.set_body(0, body);
        assert_eq!(body_handle, 1);

        // read 5 bytes
        let (chunk, flag) = context.read_body(body_handle, 5).await.unwrap();
        assert_eq!(chunk.len(), 5);
        assert_eq!(flag, false);

        let bytes = context.read_body_all(body_handle).await.unwrap();
        assert_eq!(bytes.len(), 25);

        // read chunk always return flag==true
        let (chunk, flag) = context.read_body(body_handle, 100).await.unwrap();
        assert_eq!(chunk.len(), 0);
        assert_eq!(flag, true);

        // read_all returns BodyEnded error
        let bytes = context.read_body_all(body_handle).await;
        assert!(bytes.is_err());
        assert!(matches!(bytes.err().unwrap(), super::BodyError::ReadEnded));
    }
}
