use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;

/// rand_string generates a random string of the given size
pub fn rand_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

/// obj_hash generate a hash of the given serializable object
pub fn obj_hash(value: impl Serialize) -> Result<String> {
    // serialize to json and md5 hash it
    let content = serde_json::to_string(&value)?;
    Ok(format!("{:x}", md5::compute(content)))
}

/// get hostname
pub fn get_hostname() -> Result<String> {
    // get env HOSTNAME first
    let mut h = std::env::var("HOSTNAME").unwrap_or_else(|_| "".to_string());
    if h.is_empty() {
        h = hostname::get().unwrap().to_str().unwrap().to_string();
    }
    Ok(h)
}

pub mod logging;
pub mod version;
