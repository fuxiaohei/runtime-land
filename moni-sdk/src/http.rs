pub struct Body {
    /// body_handle is an index to the body in the wasi context
    body_handle: u32,
}

impl Body {
    pub fn new(body_handle: u32) -> Self {
        Self { body_handle }
    }
    pub fn body_handle(&self) -> u32 {
        self.body_handle
    }
}

pub type Request = http::Request<Body>;
pub type Response = http::Response<Body>;
