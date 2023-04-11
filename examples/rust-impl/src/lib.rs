use moni_sdk::http::{Body, Request, Response};
use moni_sdk::http_main;

#[http_main]
pub fn handle_request(req: Request) -> Response {
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();
    let body = req.body();
    body.read(1024);
    http::Response::builder()
        .status(200)
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Body::new(body.body_handle()))
        .unwrap()
}
