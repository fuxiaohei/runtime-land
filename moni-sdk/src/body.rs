use self::http_body::{
    http_body_new, http_body_new_stream, http_body_read, http_body_read_all, http_body_write,
    HttpBodyHandle,
};
use anyhow::Result;

include!("../../wit/http_body.rs");

/// Body is a wrapper around the wasi http_body API
pub struct Body {
    /// body_handle is an index to the body in the wasi context
    body_handle: HttpBodyHandle,
    /// streaming
    is_streaming: bool,
}

impl Body {
    pub fn new(body_handle: u32) -> Self {
        Self {
            body_handle,
            is_streaming: false,
        }
    }
    pub fn empty() -> Self {
        let body_handle = http_body_new().unwrap();
        Body {
            body_handle,
            is_streaming: false,
        }
    }
    pub fn stream() -> Self {
        let body_handle = http_body_new_stream().unwrap();
        Body {
            body_handle,
            is_streaming: true,
        }
    }
    pub fn body_handle(&self) -> u32 {
        self.body_handle
    }
    pub fn read(&self, _size: u64) -> Result<(Vec<u8>, bool)> {
        let resp = http_body_read(self.body_handle);
        Ok(resp.unwrap())
    }
    pub fn into_bytes(&self) -> Result<Vec<u8>> {
        match http_body_read_all(self.body_handle) {
            Ok(resp) => Ok(resp),
            Err(e) => Err(e.into()),
        }
    }
    pub fn write(&self, data: &[u8]) -> Result<u64> {
        if !self.is_streaming {
            return Err(anyhow::anyhow!("body is not streaming"));
        }
        let resp = http_body_write(self.body_handle, data);
        Ok(resp.unwrap())
    }
    pub fn write_str(&self, data: &str) -> Result<u64> {
        if !self.is_streaming {
            return Err(anyhow::anyhow!("body is not streaming"));
        }
        let resp = http_body_write(self.body_handle, data.as_bytes());
        Ok(resp.unwrap())
    }
    pub fn is_streaming(&self) -> bool {
        self.is_streaming
    }
}

impl From<&[u8]> for Body {
    fn from(s: &[u8]) -> Self {
        let body_handle = http_body_new().unwrap();
        http_body_write(body_handle, s).unwrap();
        Body::new(body_handle)
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        let body_handle = http_body_new().unwrap();
        http_body_write(body_handle, s.as_bytes()).unwrap();
        Body::new(body_handle)
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        let body_handle = http_body_new().unwrap();
        http_body_write(body_handle, s.as_bytes()).unwrap();
        Body::new(body_handle)
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        let body_handle = http_body_new().unwrap();
        http_body_write(body_handle, v.as_slice()).unwrap();
        Body::new(body_handle)
    }
}
