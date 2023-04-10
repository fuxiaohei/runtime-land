use self::http_outgoing::{Request, RequestError, Response};
use self::http_types::{BodyError, HttpBodyHandle, RedirectPolicy, RequestOptions};
use axum::body::Body;
use reqwest::redirect;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

wasmtime::component::bindgen!({
    world:"http-interface",
    path: "../wit",
    async: true,
});

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions {
            timeout_ms: Some(30000),
            redirect: RedirectPolicy::Follow,
            redirect_limit: Some(20),
        }
    }
}

pub struct HttpImplContext {
    /// req_id set related request id from main request
    pub req_id: u64,
    /// counter is used to count fetch times, limit 10. avoid creating too many requests
    pub counter: u16,
    /// body hash map
    body_map: HashMap<u32, Body>,
    /// atomic increment id for body
    body_id: AtomicU32,
}

impl HttpImplContext {
    pub fn new(req_id: u64) -> Self {
        HttpImplContext {
            req_id,
            counter: 10,
            body_map: HashMap::new(),
            body_id: AtomicU32::new(1),
        }
    }
}

impl HttpImplContext {
    pub fn set_body(&mut self, body: Body) -> u32 {
        let id = self
            .body_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.body_map.insert(id, body);
        id
    }
}

impl TryFrom<RedirectPolicy> for redirect::Policy {
    type Error = anyhow::Error;
    fn try_from(value: RedirectPolicy) -> Result<Self, Self::Error> {
        match value {
            RedirectPolicy::Follow => Ok(redirect::Policy::default()),
            RedirectPolicy::Error => Ok(redirect::Policy::custom(|attempt| {
                attempt.error(anyhow::anyhow!("redirect policy is error"))
            })),
            RedirectPolicy::Manual => Ok(redirect::Policy::none()),
        }
    }
}

#[async_trait::async_trait]
impl http_outgoing::Host for HttpImplContext {
    async fn fetch(
        &mut self,
        _req: Request,
        _options: RequestOptions,
    ) -> wasmtime::Result<Result<Response, RequestError>> {
        return Ok(Err(RequestError::InvalidUrl("invalid url".to_string())));
    }
}

#[async_trait::async_trait]
impl http_types::Host for HttpImplContext {
    async fn http_body_read(
        &mut self,
        handle: HttpBodyHandle,
        _size: u64,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), BodyError>> {
        let body = self.body_map.get_mut(&handle).unwrap();
        Ok(Err(BodyError {}))
    }

    async fn http_body_write(
        &mut self,
        _handle: HttpBodyHandle,
        _data: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, BodyError>> {
        Ok(Err(BodyError {}))
    }

    async fn http_body_new(&mut self) -> wasmtime::Result<Result<u32, BodyError>> {
        Ok(Err(BodyError {}))
    }
}
