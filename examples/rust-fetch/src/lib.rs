use moni_sdk::http::{fetch, Body, Request, RequestOptions, Response};
use moni_sdk::http_main;

#[http_main]
pub fn handle_request(_req: Request) -> Response {
    let fetch_request = http::Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .body(Body::from(""))
        .unwrap();
    let fetch_response = fetch(fetch_request, RequestOptions::default()).unwrap();
    http::Response::builder()
        .status(fetch_response.status())
        .body(fetch_response.into_body())
        .unwrap()
}
