wasmtime::component::bindgen!({
    world:"http-fetch",
    path: "../wit/http-fetch.wit",
    async: true,
});

use http_fetch::{FetchError, FetchOptions, RedirectPolicy, Request, Response};
use reqwest::redirect;
use std::str::FromStr;
use tracing::{debug, instrument, warn};

impl Default for FetchOptions {
    fn default() -> Self {
        FetchOptions {
            timeout: 30,
            redirect: RedirectPolicy::Follow,
        }
    }
}

/// FetchCtx is fetch implmentation context
pub struct FetchCtx {
    /// req_id set related request id from main request
    pub req_id: u64,
    /// counter is used to count fetch times, limit 10. avoid creating too many requests
    pub counter: u16,
}

impl FetchCtx {
    pub fn new(req_id: u64) -> Self {
        FetchCtx { req_id, counter: 10 }
    }
}

impl TryFrom<http_fetch::RedirectPolicy> for redirect::Policy {
    type Error = anyhow::Error;
    fn try_from(value: http_fetch::RedirectPolicy) -> Result<Self, Self::Error> {
        match value {
            http_fetch::RedirectPolicy::Follow => Ok(redirect::Policy::default()),
            http_fetch::RedirectPolicy::Error => Ok(redirect::Policy::custom(|attempt| {
                attempt.error(anyhow::anyhow!("redirect policy is error"))
            })),
            http_fetch::RedirectPolicy::Manual => Ok(redirect::Policy::none()),
        }
    }
}

#[async_trait::async_trait]
impl http_fetch::HttpFetch for FetchCtx {
    #[instrument(skip_all, name = "[Fetch]", level = "debug", fields(req_id = self.req_id, counter = self.counter))]
    async fn fetch(
        &mut self,
        request: Request,
        options: FetchOptions,
    ) -> anyhow::Result<std::result::Result<Response, FetchError>> {
        debug!("{} {}", request.method, request.uri);

        self.counter += 1;

        let fetch_body = match request.body {
            Some(b) => b,
            None => vec![],
        };

        let client = reqwest::Client::builder()
            .redirect(options.redirect.try_into()?)
            .build()?;
        let fetch_response = match client
            .request(
                reqwest::Method::from_str(request.method.as_str()).unwrap(),
                request.uri.clone(),
            )
            .timeout(std::time::Duration::from_secs(options.timeout as u64))
            .body(reqwest::Body::from(fetch_body))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("failed: {e}");
                return Ok(Err(FetchError::InvalidRequest));
            }
        };

        let mut resp_headers = vec![];
        for (key, value) in fetch_response.headers() {
            resp_headers.push((key.to_string(), value.to_str().unwrap().to_string()));
        }
        let status = fetch_response.status().as_u16();
        let body = fetch_response.bytes().await?;
        let resp = Response {
            status,
            headers: resp_headers,
            body: Some(body.to_vec()),
        };
        debug!("response: {}, len={}", resp.status, body.len());
        Ok(Ok(resp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_fetch_impl() {
        use http_fetch::HttpFetch;
        let mut fetch_impl = FetchCtx::new(0);
        let req = Request {
            method: "GET".to_string(),
            uri: "https://www.rust-lang.org".to_string(),
            headers: vec![],
            body: None,
        };
        let resp = fetch_impl
            .fetch(req, FetchOptions::default())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resp.status, 200);
    }
}
