use bytes::Bytes;
use moni_sdk::http::{Request, Response};
use moni_sdk::http_main;

#[http_main]
pub fn handle_http_request(mut req: Request) -> Response {
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();
    http::Response::builder()
        .status(200)
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Bytes::from("Hello, World"))
        .unwrap()
}
