include!("../../wit/kv_storage.rs");

/// Key is the key of the key-value pair.
pub type Key = String;
/// Value is the value of the key-value pair.
pub type Value = Vec<u8>;
/// Pair is a tuple of Key and Value.
pub type Pair = (Key, Value);

/// Error is the error type for the key-value store.
pub type Error = kv_storage::KvError;

/// get returns the value of the key-value pair.
pub fn get(k: Key) -> Result<Value, Error> {
    kv_storage::get(&k)
}

/// set sets the value of the key-value pair with an expiration time.
pub fn set(k: Key, v: Value, expire: u64) -> Result<(), Error> {
    kv_storage::set(&k, &v, expire)
}

/// delete deletes the key-value pair. if not exist, it does nothing.
pub fn delete(k: Key) -> Result<(), Error> {
    kv_storage::delete(&k)
}

/// get all key-value pairs.
pub fn get_all() -> Result<Vec<Pair>, Error> {
    kv_storage::get_all()
}
