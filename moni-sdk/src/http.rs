use crate::body::Body as RawBody;

// re-export body into http crate
pub type Body = RawBody;
pub type Request = http::Request<Body>;
pub type Response = http::Response<Body>;
