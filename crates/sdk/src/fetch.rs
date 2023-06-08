use super::http::{Body, Request, Response};
use super::host::moni::http::http_outgoing::{fetch_request, RequestError, RequestOptions};
use super::host::moni::http::http_types::{self, RedirectPolicy};

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions {
            timeout: 30,
            redirect: RedirectPolicy::Follow,
        }
    }
}

pub fn fetch(req: Request, options: RequestOptions) -> Result<Response, RequestError> {
    let mut headers = vec![];
    for (key, value) in req.headers() {
        headers.push((key.to_string(), value.to_str().unwrap().to_string()));
    }
    let uri = req.uri().clone().to_string();
    let method = req.method().clone();

    let fetch_req = http_types::Request {
        headers,
        uri,
        method: method.to_string(),
        body: Some(req.body().body_handle()),
    };
    let fetch_resp = fetch_request(&fetch_req, options)?;

    let mut builder = http::Response::builder().status(fetch_resp.status);
    for (key, value) in fetch_resp.headers {
        builder = builder.header(key, value);
    }
    let resp_body = Body::new(fetch_resp.body.unwrap());
    let resp = builder.body(resp_body).unwrap();

    Ok(resp)
}
