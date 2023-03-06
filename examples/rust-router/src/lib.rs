use bytes::Bytes;
use moni_sdk::http::{router, Request, Response};
use moni_sdk::http_main;

#[http_main]
pub fn handle_http_request(mut req: Request) -> Response {
    router::get("/hello", echo_hello).unwrap();
    router::get("/foo/bar", echo_foo_bar).unwrap();
    router::get("/params/:value", echo_params).unwrap();
    router::route(req)
}

pub fn echo_hello(_req: Request) -> Response {
    http::Response::builder()
        .status(200)
        .body(Bytes::from("Hello, World"))
        .unwrap()
}

pub fn echo_foo_bar(_req: Request) -> Response {
    http::Response::builder()
        .status(200)
        .body(Bytes::from("Foo Bar"))
        .unwrap()
}

pub fn echo_params(req: Request) -> Response {
    let value = router::params(&req, "value".to_string()).unwrap();
    http::Response::builder()
        .status(200)
        .body(Bytes::from(format!("value: {value}")))
        .unwrap()
}
