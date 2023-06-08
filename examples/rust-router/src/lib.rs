use lol_sdk::http::{Body, Request, Response};
use lol_sdk::http_main;
use lol_sdk::router;

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
        .body(Body::from("Hello, World"))
        .unwrap()
}

pub fn echo_foo_bar(_req: Request) -> Response {
    http::Response::builder()
        .status(200)
        .body(Body::from("Foo Bar"))
        .unwrap()
}

pub fn echo_params(req: Request) -> Response {
    let value = router::params(&req, "value".to_string()).unwrap();
    http::Response::builder()
        .status(200)
        .body(Body::from(format!("value: {value}")))
        .unwrap()
}
