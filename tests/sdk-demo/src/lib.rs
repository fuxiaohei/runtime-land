use land_sdk::http::{Body, Error, Request, Response};
use land_sdk::http_main;
use land_sdk::router;

#[http_main]
pub fn handle_http_request(mut req: Request) -> Result<Response, Error> {
    router::get("/hello", echo_hello).unwrap();
    router::post("/foo/bar", echo_foo_bar).unwrap();
    router::any("/params/:value", echo_params).unwrap();
    router::route(req)
}

pub fn echo_hello(_req: Request) -> Result<Response, Error> {
    Ok(http::Response::builder()
        .status(200)
        .body(Body::from("Hello, World"))?)
}

pub fn echo_foo_bar(req: Request) -> Result<Response, Error> {
    let body = req.body().to_bytes()?;
    Ok(http::Response::builder()
        .status(200)
        .body(Body::from(format!("Foo Bar, BodySize: {}", body.len())))?)
}

pub fn echo_params(req: Request) -> Result<Response, Error> {
    let value = router::params(&req, "value".to_string()).unwrap_or_default();
    Ok(http::Response::builder()
        .status(200)
        .body(Body::from(format!("value: {value}")))?)
}
