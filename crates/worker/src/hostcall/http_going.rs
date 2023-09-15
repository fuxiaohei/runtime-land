use super::host::land::http::http_outgoing::{
    Host, Request, RequestError, RequestOptions, Response,
};
use super::host::land::http::http_types::RedirectPolicy;
use super::HttpContext;
use hyper::Body;
use reqwest::redirect;
use std::str::FromStr;
use tracing::{debug, info, instrument, warn};

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions {
            timeout: 30,
            redirect: RedirectPolicy::Follow,
        }
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
impl Host for HttpContext {
    #[instrument(skip_all, name = "[Fetch]", level = "warn", fields(req_id = self.req_id, counter = self.fetch_counter))]
    async fn fetch_request(
        &mut self,
        request: Request,
        options: RequestOptions,
    ) -> anyhow::Result<std::result::Result<Response, RequestError>> {
        debug!("{} {}", request.method, request.uri);

        let body = match request.body {
            Some(body_handle) => self.take_body(body_handle).unwrap(),
            None => Body::empty(),
        };

        info!("fetch: {} {}", request.method, request.uri);

        // use client pool to reuse client
        let client = HttpContext::get_client(options.redirect);

        let fetch_response = match client
            .request(
                reqwest::Method::from_str(request.method.as_str()).unwrap(),
                request.uri.clone(),
            )
            .timeout(std::time::Duration::from_secs(options.timeout as u64))
            .body(body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("failed: {e}");
                let content = e.to_string();
                if content.contains("connect") {
                    return Ok(Err(RequestError::NetworkError));
                }
                return Ok(Err(RequestError::InvalidRequest));
            }
        };

        let mut resp_headers = vec![];
        let mut is_stream = true;
        for (key, value) in fetch_response.headers() {
            if key == "content-length" {
                is_stream = false;
            }
            resp_headers.push((key.to_string(), value.to_str().unwrap().to_string()));
        }
        let status = fetch_response.status().as_u16();
        let body_handle = if is_stream {
            let body_stream = fetch_response.bytes_stream();
            let body = Body::wrap_stream(body_stream);
            self.set_body(body)
        } else {
            let body = fetch_response.bytes().await.unwrap();
            let body = Body::from(body);
            self.set_body(body)
        };
        let resp = Response {
            status,
            headers: resp_headers,
            body: Some(body_handle),
        };
        debug!("response: {}, handle={}", resp.status, body_handle);
        Ok(Ok(resp))
    }
}
