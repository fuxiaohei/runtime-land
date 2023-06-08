pub mod http;
pub mod router;

mod host;
mod body;
mod fetch;

/// Re-export macro from sdk-macro
pub use lol_sdk_macro::http_main;
