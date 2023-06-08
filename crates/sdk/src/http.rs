use crate::body::Body as RawBody;

// re-export body into http crate
pub type Body = RawBody;
pub type Request = http::Request<Body>;
pub type Response = http::Response<Body>;

// re-export http_outgoing into http crate
pub use super::fetch::fetch;
pub use super::host::moni::http::http_outgoing::{RequestError, RequestOptions};

pub fn error_response(status: http::StatusCode, message: String) -> Response {
    let mut response = Response::new(message.into());
    *response.status_mut() = status;
    response
}
