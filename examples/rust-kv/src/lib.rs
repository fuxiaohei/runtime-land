use bytes::Bytes;
use moni_sdk::{
    http::{error_response, Request, Response},
    http_main, kv,
};
use rand::Rng;

#[http_main]
pub fn handle_sdk_http(mut _req: Request) -> Response {
    let mut rng = rand::thread_rng();
    let rand_value: u8 = rng.gen();
    let value1 = format!("value1:{:?}", rand_value);

    /// set expire time to 0, means no expire
    kv::set("key1".to_string(), value1.as_bytes().to_vec(), 0).unwrap();

    let value2 = match kv::get("key1".to_string()) {
        Ok(v) => v,
        Err(e) => return error_response(http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Bytes::from(value2))
        .unwrap()
}
