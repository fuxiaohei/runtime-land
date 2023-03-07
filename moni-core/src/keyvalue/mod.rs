/// The Key type is a string that is used to identify a value in the key-value store.
pub type Key = String;

/// The Value type is a value with expire time that is used to store a value in the key-value store.
pub type Value = (Vec<u8>, u64);

/// The Pair type is a tuple of Key and Value.
pub type Pair = (Key, Value);

/// KvError is the error type for the key-value store.
#[derive(Debug, PartialEq)]
pub enum Error {
    KeyNotFound,
    InvalidKey,
    InternalError,
    KeyTooLarge,
    ValueTooLarge,
}

// KvStorage is the interface for the key-value store.
#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&mut self, k: Key) -> Result<Value, Error>;
    async fn set(&mut self, k: Key, v: Value) -> Result<(), Error>;
    async fn delete(&mut self, k: Key) -> Result<(), Error>;
    async fn get_all(&mut self) -> Result<Vec<Pair>, Error>;
}

impl std::fmt::Debug for dyn Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KvStorage")
    }
}

mod sled;
pub use self::sled::SledStorage;
