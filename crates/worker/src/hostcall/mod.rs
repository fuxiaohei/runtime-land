use hyper::body::Incoming;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

mod body;
mod guest;
mod host;
mod outgoing;

pub use guest::exports::land::http::incoming::{Request, Response};
pub use guest::HttpHandler;
pub use host::HttpService;

pub struct HttpContext {
    /// req_id is the unique request id for each request.
    pub req_id: String,
    /// fetch_counter is the counter for fetch.
    pub fetch_counter: u16,

    /// body_map is the map for body.
    body_map: HashMap<u32, Incoming>,
    /// body_id is increamented for each body.
    body_id: AtomicU32,
}

impl HttpContext {
    pub fn new(req_id: String) -> Self {
        Self {
            req_id,
            fetch_counter: 10,
            body_map: HashMap::new(),
            body_id: AtomicU32::new(0),
        }
    }

    pub fn take_body(&mut self, id: u32) -> Option<Incoming> {
        self.body_map.remove(&id)
    }

    pub fn set_body(&mut self, body: Incoming) -> u32 {
        let id = self
            .body_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.body_map.insert(id, body);
        id
    }

    pub fn replace_body(&mut self, id: u32, body: Incoming) -> Option<Incoming> {
        self.body_map.insert(id, body)
    }
}

impl host::land::http::types::Host for HttpContext {}
