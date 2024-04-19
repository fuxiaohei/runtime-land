mod body;
mod context;
mod fetch;
mod guest;
mod host;

pub use context::{init_clients, HttpContext};
pub use guest::exports::land::http::incoming::{Request, Response};
pub use guest::HttpHandler;
pub use host::HttpService;

impl host::land::http::types::Host for HttpContext {}
