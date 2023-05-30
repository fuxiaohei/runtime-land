wit_bindgen::generate!(in "../../wit-v2");

struct HttpServiceImpl {}

use exports::moni::moni::http_incoming;

impl http_incoming::HttpIncoming for HttpServiceImpl {
    fn handle_request(_req: http_incoming::Request) -> http_incoming::Response {
        http_incoming::Response {
            status: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: Some(1), // body handle u32
        }
    }
}

export_http_service!(HttpServiceImpl);