use super::host::moni::http::http_outgoing::{
    Host, Request, RequestError, RequestOptions, Response,
};
use super::host::moni::http::http_types::RedirectPolicy;
use super::HttpContext;
use hyper::Body;
use reqwest::redirect;
use std::str::FromStr;
use tracing::{debug, instrument, warn};

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
    #[instrument(skip_all, name = "[Fetch]", level = "debug", fields(req_id = self.req_id, counter = self.counter))]
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

        let client = reqwest::Client::builder()
            .redirect(options.redirect.try_into()?)
            .build()?;

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
                return Ok(Err(RequestError::InvalidRequest));
            }
        };

        let mut resp_headers = vec![];
        for (key, value) in fetch_response.headers() {
            resp_headers.push((key.to_string(), value.to_str().unwrap().to_string()));
        }
        let status = fetch_response.status().as_u16();
        let body_stream = fetch_response.bytes_stream();
        let body = Body::wrap_stream(body_stream);
        let body_handle = self.set_body(body);
        let resp = Response {
            status,
            headers: resp_headers,
            body: Some(body_handle),
        };
        debug!("response: {}, handle={}", resp.status, body_handle);
        Ok(Ok(resp))
    }
}
