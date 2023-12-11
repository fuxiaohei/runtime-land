use super::host::land::http::body::BodyError;
use super::host::land::http::outgoing::{Host, Request, RequestError, RequestOptions, Response};
use super::HttpContext;
use crate::hostcall::host::land::http::types::RedirectPolicy;
use axum_core::body::Body;
use bytes::Bytes;
use http_body::{Frame, SizeHint};
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Once;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::info;
use ureq::Agent;

static UREQ_AGENT_FOLLOW_REDIRECTS: OnceCell<Agent> = OnceCell::new();
static UREQ_AGENT_NO_FOLLOW_REDIRECTS: OnceCell<Agent> = OnceCell::new();
static UREQ_AGENT_ONCE: Once = Once::new();

#[async_trait::async_trait]
impl Host for HttpContext {
    async fn fetch_request(
        &mut self,
        request: Request,
        options: RequestOptions,
    ) -> wasmtime::Result<Result<Response, RequestError>> {
        // init ureq agent
        UREQ_AGENT_ONCE.call_once(|| {
            let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";
            // set ureq agent follow redirects, but limit to 10
            UREQ_AGENT_FOLLOW_REDIRECTS
                .set(ureq::AgentBuilder::new().user_agent(user_agent).redirects(10).build())
                .expect("init ureq agent failed");
            // set ureq agent no follow redirects
            UREQ_AGENT_NO_FOLLOW_REDIRECTS
                .set(ureq::AgentBuilder::new().user_agent(user_agent).redirects(0).build())
                .expect("init ureq agent failed");
        });

        info!(
            "fetch: {} {} {:?}",
            &request.method, &request.uri, request.body
        );

        let (request, body_content, body_handle) = prepare_request(
            self,
            &request,
            Duration::from_secs(options.timeout as u64),
            options.redirect == RedirectPolicy::Follow,
        )
        .await?;

        // create tx and rx to send request in tokio spawn
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn(async move {
            match send_request(request, body_content, body_handle).await {
                Ok(resp) => {
                    let _ = tx.send(Ok(resp));
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                }
            }
        });

        let (response, response_body) = rx
            .await
            .map_err(|err| RequestError::NetworkError(err.to_string()))??;
        // set body content to body handle
        self.set_body(body_handle, response_body);
        return Ok(Ok(response));
    }
}

pub struct OutgoingResponseBody {
    inner: Box<dyn std::io::Read + Send + Sync>,
    size: Option<u64>,
}

impl http_body::Body for OutgoingResponseBody {
    type Data = Bytes;
    type Error = BodyError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let mut buf = vec![0; 1 << 15]; // read 32k at a time
        match self.inner.read(&mut buf) {
            Ok(0) => Poll::Ready(None),
            Ok(n) => {
                buf.truncate(n);
                Poll::Ready(Some(Ok(Frame::data(Bytes::from(buf)))))
            }
            Err(err) => Poll::Ready(Some(Err(BodyError::ReadFailed(err.to_string())))),
        }
    }

    fn size_hint(&self) -> SizeHint {
        if let Some(size) = self.size {
            SizeHint::with_exact(size)
        } else {
            SizeHint::default()
        }
    }
}

async fn prepare_request(
    context: &mut HttpContext,
    request: &Request,
    timeout: Duration,
    follow_redirects: bool,
) -> Result<(ureq::Request, Bytes, u32), RequestError> {
    // Prepare ureq request and body content
    let request_body = match request.body {
        Some(body_handle) => context.take_body(body_handle).unwrap(),
        None => Body::empty(),
    };

    // Use ureq to send request
    let agent = if follow_redirects {
        UREQ_AGENT_FOLLOW_REDIRECTS.get().unwrap()
    } else {
        UREQ_AGENT_NO_FOLLOW_REDIRECTS.get().unwrap()
    };

    // Create ureq request
    let mut ureq_request = agent.request(request.method.as_str(), request.uri.clone().as_str());
    ureq_request = ureq_request.timeout(timeout);
    for (name, value) in &request.headers {
        ureq_request = ureq_request.set(name, value);
    }
    let body_content = axum::body::to_bytes(request_body, usize::MAX)
        .await
        .map_err(|err| RequestError::InvalidRequest(err.to_string()))?;

    // Create a body handle placeholder with an empty body
    let body_handle = context.set_body(0, Body::empty());

    Ok((ureq_request, body_content, body_handle))
}

async fn send_request(
    request: ureq::Request,
    body_content: Bytes,
    body_handle: u32,
) -> Result<(Response, Body), RequestError> {
    let http_response = match request.send_bytes(&body_content) {
        Ok(response) => response,
        Err(err) => return Err(RequestError::NetworkError(err.to_string())),
    };
    let status = http_response.status();
    let names = http_response.headers_names();

    // clean headers repeated
    let names = names
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut headers: Vec<(String, String)> = Vec::new();
    for name in names {
        let values = http_response.all(&name);
        for value in values {
            headers.push((name.to_string(), value.to_string()));
        }
    }
    let outgoing_body = OutgoingResponseBody {
        inner: http_response.into_reader(),
        size: None,
    };
    let response_body = Body::new(outgoing_body);
    let response = Response {
        status,
        headers,
        body: Some(body_handle),
    };
    Ok((response, response_body))
}
