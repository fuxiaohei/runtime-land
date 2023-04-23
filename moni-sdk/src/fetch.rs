include!("../../wit/http_outgoing.rs");

use self::http_outgoing::{RedirectPolicy, RequestError, RequestOptions};
use super::http::{Body, Request, Response};

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

    let fetch_req = http_outgoing::Request {
        headers: &headers,
        uri: uri.as_str(),
        method: method.as_str(),
        body: Some(req.body().body_handle()),
    };
    let fetch_resp = http_outgoing::fetch(fetch_req, options)?;

    let mut builder = http::Response::builder().status(fetch_resp.status);
    for (key, value) in fetch_resp.headers {
        builder = builder.header(key, value);
    }
    let resp_body = Body::new(fetch_resp.body.unwrap());
    let resp = builder.body(resp_body).unwrap();

    Ok(resp)
}
