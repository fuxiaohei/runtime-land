mod http_service;
mod body;
mod fetch;

pub mod http;
pub mod router;

/// Re-export macro from sdk-macro
pub use land_sdk_macro::http_main;