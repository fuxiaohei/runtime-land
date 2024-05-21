use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn rand_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn encrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>> {
    simple_crypt::encrypt(data, password)
}

pub fn decrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>> {
    simple_crypt::decrypt(data, password)
}

pub fn encrypt_text(data: &str, password: &str) -> Result<String> {
    let data = data.as_bytes();
    let password = password.as_bytes();
    let encrypted = encrypt(data, password)?;
    Ok(base64_encode(&encrypted))
}

pub fn decrypt_text(data: &str, password: &str) -> Result<String> {
    let data = base64_decode(data)?;
    let password = password.as_bytes();
    let decrypted = decrypt(&data, password)?;
    Ok(String::from_utf8(decrypted)?)
}

pub fn base64_encode(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

pub fn base64_encode_text(data: &str) -> String {
    let data = data.as_bytes();
    base64_encode(data)
}

pub fn base64_decode(data: &str) -> Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(data)?)
}

pub fn base64_decode_text(data: &str) -> Result<String> {
    let data = base64_decode(data)?;
    Ok(String::from_utf8(data)?)
}
