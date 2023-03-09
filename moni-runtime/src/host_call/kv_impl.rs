wasmtime::component::bindgen!({
    world:"kv-storage",
    path: "../wit/kv-storage.wit",
    async: true,
});

use moni_core::keyvalue::{Error, Storage};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Provider is storage impl provider
pub type Provider = Arc<Mutex<dyn Storage>>;

pub struct KvCtx {
    storage: Provider,
}

impl KvCtx {
    pub fn new(storage: Provider) -> Self {
        KvCtx { storage }
    }
}

/// convert keyvalue::Error to kv_storage::KvError
impl From<Error> for kv_storage::KvError {
    fn from(e: Error) -> Self {
        match e {
            Error::KeyNotFound => kv_storage::KvError::KeyNotFound,
            Error::InternalError => kv_storage::KvError::InternalError,
            Error::ValueTooLarge => kv_storage::KvError::ValueTooLarge,
            Error::InvalidKey => kv_storage::KvError::InvalidKey,
            Error::KeyTooLarge => kv_storage::KvError::KeyTooLarge,
        }
    }
}

#[async_trait::async_trait]
impl kv_storage::Host for KvCtx {
    async fn get(
        &mut self,
        k: kv_storage::Key,
    ) -> anyhow::Result<Result<kv_storage::Value, kv_storage::KvError>> {
        let mut store = self.storage.lock().await;
        let value = match store.get(k).await {
            Ok(v) => v,
            Err(e) => return Ok(Err(e.into())),
        };
        Ok(Ok(value.0))
    }
    async fn set(
        &mut self,
        k: kv_storage::Key,
        v: kv_storage::Value,
        expire: u64,
    ) -> anyhow::Result<Result<(), kv_storage::KvError>> {
        let mut store = self.storage.lock().await;
        match store.set(k, (v, expire)).await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => return Ok(Err(e.into())),
        }
    }
    async fn delete(
        &mut self,
        k: kv_storage::Key,
    ) -> anyhow::Result<Result<(), kv_storage::KvError>> {
        let mut store = self.storage.lock().await;
        match store.delete(k).await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => return Ok(Err(e.into())),
        }
    }
    async fn get_all(
        &mut self,
    ) -> anyhow::Result<Result<Vec<kv_storage::Pair>, kv_storage::KvError>> {
        let mut store = self.storage.lock().await;
        let values = match store.get_all().await {
            Ok(v) => v,
            Err(e) => return Ok(Err(e.into())),
        };
        let mut pairs = Vec::new();
        for (k, (v, _expire)) in values {
            pairs.push((k, v));
        }
        Ok(Ok(pairs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kv_impl::kv_storage::Host;

    #[tokio::test]
    async fn run_kv_storage_impl() {
        let storage = moni_core::keyvalue::SledStorage::new("keyvalue-test").unwrap();
        let storage = Arc::new(Mutex::new(storage));
        let mut kv_storage = KvCtx::new(storage);
        kv_storage
            .set("abc".to_string(), "abcd".as_bytes().to_vec(), 100)
            .await
            .unwrap()
            .unwrap();
        let value = kv_storage.get("abc".to_string()).await.unwrap().unwrap();
        assert_eq!(value, "abcd".as_bytes().to_vec());

        // get not exist key
        let value = kv_storage.get("not_exist".to_string()).await.unwrap();
        assert!(value.is_err());

        let values = kv_storage.get_all().await.unwrap().unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, "abc");
        assert_eq!(values[0].1, "abcd".as_bytes().to_vec());

        kv_storage.delete("abc".to_string()).await.unwrap().unwrap();
        let values = kv_storage.get_all().await.unwrap().unwrap();
        assert_eq!(values.len(), 0);
    }
}
