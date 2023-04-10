use moni_sdk::export_http_interface;
use moni_sdk::http::{Body, Request, Response};
use moni_sdk::wit::http_incoming;

pub fn handle_request(req: Request) -> Response {
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();
    let body = req.body();
    http::Response::builder()
        .status(200)
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Body::new(body.body_handle()))
        .unwrap()
}

struct HttpInterface;

impl http_incoming::HttpIncoming for HttpInterface {
    fn handle_request(req: http_incoming::Request) -> http_incoming::Response {
        // convert wasm_request to sdk_request
        let sdk_request: Request = req.try_into().unwrap();
        let sdk_response = handle_request(sdk_request);

        let sdk_response_body_handle = sdk_response.body().body_handle();
        // convert sdk_response to wasm_response
        match sdk_response.try_into() {
            Ok(r) => r,
            Err(_e) => http_incoming::Response {
                status: 500,
                headers: vec![],
                body: Some(sdk_response_body_handle),
            },
        }
    }
}

export_http_interface!(HttpInterface);
