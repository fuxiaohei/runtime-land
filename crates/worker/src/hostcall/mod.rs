mod body;
mod context;
mod guest;
mod host;
mod outgoing;

pub use context::HttpContext;
pub use guest::exports::land::http::incoming::{Request, Response};
pub use guest::HttpHandler;
pub use host::HttpService;

impl host::land::http::types::Host for HttpContext {}

#[cfg(test)]
mod tests {
    use super::HttpContext;
    use crate::hostcall::host::land::http::body::BodyError;
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
        assert!(matches!(bytes.err().unwrap(), BodyError::ReadClosed));
    }

    #[tokio::test]
    async fn test_http_context_body_write() {
        let mut context = HttpContext::new("test-id".to_string());
        let handle = context.new_body();
        let data = "abc".repeat(10);
        let size = context
            .write_body(handle, data.as_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(size, 30);

        let (chunk, flag) = context.read_body(handle, 10).await.unwrap();
        assert_eq!(chunk.len(), 10);
        assert_eq!(flag, false);

        let all = context.read_body_all(handle).await.unwrap();
        assert_eq!(all.len(), 20);

        let (chunk, flag) = context.read_body(handle, 10).await.unwrap();
        assert_eq!(chunk.len(), 0);
        assert_eq!(flag, true);
    }

    #[tokio::test]
    async fn test_http_context_body_write_stream() {
        let mut context = HttpContext::new("test-id".to_string());
        let handle = context.new_body_stream();
        let data = "abc".repeat(10);
        let size = context
            .write_body(handle, data.as_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(size, 30);

        // channel length is 3, so we can write 3 times
        let size = context
            .write_body(handle, data.as_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(size, 30);
        let size = context
            .write_body(handle, data.as_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(size, 30);

        // write 4th time will return error
        let size = context.write_body(handle, data.as_bytes().to_vec()).await;
        assert!(size.is_err());
        assert!(matches!(size.err().unwrap(), BodyError::WriteFailed(_)));

        // read 10 bytes out, channel left 2. we can write again
        let (chunk, flag) = context.read_body(handle, 10).await.unwrap();
        assert_eq!(chunk.len(), 10);
        assert_eq!(flag, false);

        let size = context
            .write_body(handle, data.as_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(size, 30);

        // body read all done, it can't write again
        let all = context.read_body_all(handle).await.unwrap();
        assert_eq!(all.len(), 110);

        let size = context.write_body(handle, data.as_bytes().to_vec()).await;
        assert!(size.is_err());
        assert!(matches!(size.err().unwrap(), BodyError::WriteClosed));
    }
}
