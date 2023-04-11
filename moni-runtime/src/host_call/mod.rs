use hyper::body::{Body, Sender};
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

pub struct HttpContext {
    /// req_id set related request id from main request
    pub req_id: u64,
    /// counter is used to count fetch times, limit 10. avoid creating too many requests
    pub counter: u16,
    /// body hash map
    body_map: HashMap<u32, Body>,
    /// body sender
    body_sender_map: HashMap<u32, Sender>,
    /// atomic increment id for body
    body_id: AtomicU32,
}

impl HttpContext {
    pub fn new(req_id: u64) -> Self {
        HttpContext {
            req_id,
            counter: 10,
            body_map: HashMap::new(),
            body_sender_map: HashMap::new(),
            body_id: AtomicU32::new(1),
        }
    }
    pub fn set_body(&mut self, body: Body) -> u32 {
        let id = self
            .body_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.body_map.insert(id, body);
        id
    }

    fn set_body_sender(&mut self, id: u32, sender: Sender) {
        self.body_sender_map.insert(id, sender);
    }
}

pub mod http_body;
pub mod http_incoming;
