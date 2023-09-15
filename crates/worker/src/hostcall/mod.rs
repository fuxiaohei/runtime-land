mod guest;
pub use guest::exports::land::http::http_incoming::{Request, Response};
pub use guest::HttpHandler;

mod host;
pub use host::HttpService;

pub mod http_body;
pub mod http_going;
pub mod http_types;

use self::host::land::http::http_types::RedirectPolicy;
use hyper::body::{Body, Sender};
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Once;

static CLIENT_INIT_ONCE: Once = Once::new();

static REDIRECT_FOLLOW_POOL: OnceCell<Client> = OnceCell::new();
static REDIRECT_ERROR_POOL: OnceCell<Client> = OnceCell::new();
static REDIRECT_MANUAL_POOL: OnceCell<Client> = OnceCell::new();

pub struct HttpContext {
    /// req_id set related request id from main request
    pub req_id: String,
    /// fetch_counter is used to count fetch times, limit 10. avoid creating too many requests
    pub fetch_counter: u16,
    /// body hash map
    body_map: HashMap<u32, Body>,
    /// body sender
    body_sender_map: HashMap<u32, Sender>,
    /// atomic increment id for body
    body_id: AtomicU32,
}

impl HttpContext {
    pub fn get_client(r: RedirectPolicy) -> Client {
        match r {
            RedirectPolicy::Follow => REDIRECT_FOLLOW_POOL.get().unwrap().clone(),
            RedirectPolicy::Error => REDIRECT_ERROR_POOL.get().unwrap().clone(),
            RedirectPolicy::Manual => REDIRECT_MANUAL_POOL.get().unwrap().clone(),
        }
    }
    pub fn new(req_id: String) -> Self {
        CLIENT_INIT_ONCE.call_once(|| {
            REDIRECT_ERROR_POOL
                .set(
                    reqwest::Client::builder()
                        .redirect(RedirectPolicy::Error.try_into().unwrap())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            REDIRECT_FOLLOW_POOL
                .set(
                    reqwest::Client::builder()
                        .redirect(RedirectPolicy::Follow.try_into().unwrap())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            REDIRECT_MANUAL_POOL
                .set(
                    reqwest::Client::builder()
                        .redirect(RedirectPolicy::Manual.try_into().unwrap())
                        .build()
                        .unwrap(),
                )
                .unwrap();
        });
        HttpContext {
            req_id,
            fetch_counter: 10,
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

    pub fn replace_body(&mut self, id: u32, body: Body) -> Option<Body> {
        self.body_map.insert(id, body)
    }

    pub fn take_body(&mut self, id: u32) -> Option<Body> {
        self.body_map.remove(&id)
    }

    fn set_body_sender(&mut self, id: u32, sender: Sender) {
        self.body_sender_map.insert(id, sender);
    }
}
