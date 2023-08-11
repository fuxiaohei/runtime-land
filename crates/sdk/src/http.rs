//! `http` is a module for http request and response.

use crate::body::Body as RawBody;

/// `Body` is a wrapper around the wasi http_body API
pub type Body = RawBody;
/// `Request` is a wrapper around the wasi http_request API
pub type Request = http::Request<Body>;
/// `Response` is a wrapper around the wasi http_response API
pub type Response = http::Response<Body>;

// re-export http_outgoing into http crate
pub use super::fetch::fetch;

/// `RequestError` is error type when fetching request failed.
pub type RequestError = super::http_service::land::http::http_outgoing::RequestError;
/// `RequestOptions` is options for fetching request, including timeout and redirect policy.
pub type RequestOptions = super::http_service::land::http::http_outgoing::RequestOptions;

/// `error_response` is a helper function to build a response with status code and message.
pub fn error_response(status: http::StatusCode, message: String) -> Response {
    let mut response = Response::new(message.into());
    *response.status_mut() = status;
    response
}
