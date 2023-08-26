use land_sdk::{
    http::{Body, Error, Request, Response},
    http_main,
};

#[http_main]
pub fn handle_request(req: Request) -> Result<Response, Error> {
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();
    Ok(http::Response::builder()
        .status(200)
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Body::from("Hello Runtime.land!!"))
        .unwrap())
}
