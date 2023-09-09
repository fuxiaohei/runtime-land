use once_cell::sync::OnceCell;

/// ENDPOINT is the name of endpoint
pub static ENDPOINT: OnceCell<String> = OnceCell::new();