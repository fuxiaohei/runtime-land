use crate::http::{Body, Request, Response};

include!("../../wit/http_interface.rs");

impl TryFrom<http_incoming::Request> for Request {
    type Error = anyhow::Error;

    fn try_from(wasm_req: http_incoming::Request) -> Result<Self, Self::Error> {
        use std::str::FromStr;

        let mut http_req = http::Request::builder()
            .method(http::Method::from_str(wasm_req.method.as_str())?)
            .uri(&wasm_req.uri);

        for (key, value) in wasm_req.headers {
            http_req = http_req.header(key, value);
        }
        // 1 is the request body handle, which is defined in wasi host functions
        let body = Body::new(wasm_req.body.unwrap_or(1));
        Ok(http_req.body(body)?)
    }
}

impl TryFrom<Response> for http_incoming::Response {
    type Error = anyhow::Error;

    fn try_from(http_res: Response) -> Result<Self, Self::Error> {
        let status = http_res.status().as_u16();
        let mut headers: Vec<(String, String)> = vec![];
        for (key, value) in http_res.headers() {
            headers.push((key.to_string(), value.to_str()?.to_string()));
        }
        let body = http_res.body();
        Ok(http_incoming::Response {
            status,
            headers,
            body: Some(body.body_handle()),
        })
    }
}
