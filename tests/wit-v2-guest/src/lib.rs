wit_bindgen::generate!("http-handler" in "../../wit");

struct HttpServiceImpl {}

use exports::land::http::http_incoming;

impl http_incoming::HttpIncoming for HttpServiceImpl {
    fn handle_request(_req: http_incoming::Request) -> http_incoming::Response {
        http_incoming::Response {
            status: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: Some(1), // body handle u32
        }
    }
}

export_http_handler!(HttpServiceImpl);
