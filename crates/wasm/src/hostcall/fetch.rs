use super::host::land::http::outgoing::{Host, Request, RequestError, RequestOptions, Response};
use super::host::land::http::types::RedirectPolicy;
use super::HttpContext;
use axum::body::Body;
use reqwest::redirect;
use std::str::FromStr;
use tracing::{debug, warn};

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
    async fn fetch_request(
        &mut self,
        request: Request,
        options: RequestOptions,
    ) -> wasmtime::Result<Result<Response, RequestError>> {
        let st = tokio::time::Instant::now();
        debug!(method = request.method, uri = request.uri, "Fetch start");

        // use client pool to reuse client
        let client = HttpContext::get_http_client(options.redirect);

        // read body
        let body = match self.take_body(request.body.unwrap_or(0)) {
            Some(b) => b,
            None => Body::empty(),
        };

        // read body bytes,
        // TODO: use streaming way to read body
        let body_bytes = axum::body::to_bytes(body, std::usize::MAX).await?;

        // call fetch
        let fetch_response = match client
            .request(
                reqwest::Method::from_str(request.method.as_str()).unwrap(),
                request.uri.clone(),
            )
            .timeout(std::time::Duration::from_secs(options.timeout as u64))
            .body(body_bytes)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    method = request.method,
                    uri = request.uri,
                    "Fetch failed: {e}"
                );
                let content = e.to_string();
                if content.contains("connect") {
                    return Ok(Err(RequestError::NetworkError(e.to_string())));
                }
                return Ok(Err(RequestError::InvalidRequest(e.to_string())));
            }
        };

        let mut resp_headers = vec![];
        // if body is stream, header should not contain content-length, use Transfer-Encoding:chunk
        let mut is_stream = true;
        let mut content_length: usize = 0;
        for (key, value) in fetch_response.headers() {
            if key == "content-length" {
                is_stream = false;
                content_length = value.to_str().unwrap().parse().unwrap();
            }
            let header_value = String::from_utf8_lossy(value.as_bytes()).to_string();
            resp_headers.push((key.to_string(), header_value));
        }

        let status = fetch_response.status().as_u16();
        let body_handle = if is_stream {
            let body_stream = fetch_response.bytes_stream();
            let body = Body::from_stream(body_stream);
            self.set_body(0, body)
        } else {
            let body = fetch_response.bytes().await.unwrap();
            let body = Body::from(body);
            self.set_body(0, body)
        };
        debug!(
            method = request.method,
            uri = request.uri,
            "Fetch set body: {}, is_stream:{}, content_length:{}",
            body_handle,
            is_stream,
            content_length,
        );
        let resp = Response {
            status,
            headers: resp_headers,
            body: Some(body_handle),
        };
        let elasped = st.elapsed().as_millis();
        debug!(
            method = request.method,
            uri = request.uri,
            status = resp.status,
            handle = body_handle,
            cost = elasped,
            "Fetch done",
        );
        Ok(Ok(resp))
    }
}
