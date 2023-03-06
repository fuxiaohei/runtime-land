mod fetch;

pub mod http {
    use bytes::Bytes;

    pub type Request = http::Request<Bytes>;
    pub type Response = http::Response<Bytes>;

    pub fn error_response(status: http::StatusCode, message: String) -> Response {
        let mut response = Response::new(message.into());
        *response.status_mut() = status;
        response
    }

    pub type Error = super::fetch::FetchError;
    pub use super::fetch::{fetch, FetchOptions, RedirectPolicy};
}

/// Re-export macro from sdk-macro
pub use moni_sdk_macro::http_main;
