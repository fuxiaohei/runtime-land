use super::{Error, Key, Pair, Storage, Value};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// MEMORY_KV_VALUE_MAX_SIZE is the maximum size of the value in the memory key-value store.
const MEMORY_KV_VALUE_MAX_SIZE: usize = 1024 * 1024;
/// MEMORY_KV_KEY_MAX_SIZE is the maximum size of the key in the memory key-value store.
const MEMORY_KV_KEY_MAX_SIZE: usize = 1024;

fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug)]
pub struct SledStorage {
    db: sled::Db,
}

impl SledStorage {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ValueItem {
    value: Vec<u8>,
    expire: u64,
}

#[async_trait::async_trait]
impl Storage for SledStorage {
    async fn get(&mut self, k: Key) -> Result<Value, Error> {
        let value = self
            .db
            .get(k.as_bytes())
            .map_err(|_| Error::KeyNotFound)?
            .ok_or(Error::KeyNotFound)?
            .to_vec();
        let item: ValueItem = bincode::deserialize(&value).map_err(|_| Error::InternalError)?;
        if item.expire > 0 && item.expire < get_current_timestamp() {
            // value is expired
            return Err(Error::KeyNotFound);
        }
        Ok((item.value, item.expire))
    }
    async fn set(&mut self, k: Key, v: Value) -> Result<(), Error> {
        if k.len() > MEMORY_KV_KEY_MAX_SIZE {
            return Err(Error::KeyTooLarge);
        }
        if v.0.len() > MEMORY_KV_VALUE_MAX_SIZE {
            return Err(Error::ValueTooLarge);
        }
        let mut item = ValueItem {
            value: v.0,
            expire: v.1,
        };
        // set expire time
        if item.expire > 0 {
            item.expire += get_current_timestamp();
        }
        let content = bincode::serialize(&item).map_err(|_| Error::InternalError)?;
        self.db
            .insert(k.as_bytes(), content)
            .map_err(|_| Error::InternalError)?;
        Ok(())
    }
    async fn delete(&mut self, k: Key) -> Result<(), Error> {
        self.db
            .remove(k.as_bytes())
            .map_err(|_| Error::InternalError)?;
        Ok(())
    }
    async fn get_all(&mut self) -> Result<Vec<Pair>, Error> {
        let mut pairs = vec![];
        let iter = self.db.iter();
        for item in iter {
            let item = item.map_err(|_| Error::InternalError)?;
            let value_item: ValueItem =
                bincode::deserialize(&item.1).map_err(|_| Error::InternalError)?;
            if value_item.expire > 0 && value_item.expire < get_current_timestamp() {
                // value is expired
                continue;
            }
            let pair: Pair = (
                String::from_utf8(item.0.to_vec()).unwrap(),
                (value_item.value, value_item.expire),
            );
            pairs.push(pair);
        }
        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_sled() {
        let mut sled = SledStorage::new("keyvalue-test").unwrap();

        // set and get
        sled.set("abc".to_string(), ("abcabc".as_bytes().to_vec(), 0))
            .await
            .unwrap();
        let value = sled.get("abc".to_string()).await.unwrap();
        assert_eq!(value.0, "abcabc".as_bytes().to_vec());
        assert_eq!(value.1, 0);

        // set expired
        sled.set("xyz".to_string(), ("xyzxyz".as_bytes().to_vec(), 1))
            .await
            .unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let value = sled.get("xyz".to_string()).await;
        assert_eq!(value.err().unwrap(), Error::KeyNotFound);

        // set and delete
        sled.set("123".to_string(), ("123123".as_bytes().to_vec(), 0))
            .await
            .unwrap();
        let value = sled.get("123".to_string()).await.unwrap();
        assert_eq!(value.0, "123123".as_bytes().to_vec());
        sled.delete("123".to_string()).await.unwrap();
        let value = sled.get("123".to_string()).await;
        assert_eq!(value.err().unwrap(), Error::KeyNotFound);

        // get_all
        sled.set("abc1".to_string(), ("abcabc1".as_bytes().to_vec(), 0))
            .await
            .unwrap();
        sled.set("abc2".to_string(), ("abcabc2".as_bytes().to_vec(), 0))
            .await
            .unwrap();
        sled.set("abc3".to_string(), ("abcabc3".as_bytes().to_vec(), 0))
            .await
            .unwrap();
        let pairs = sled.get_all().await.unwrap();
        assert_eq!(pairs.len(), 4); // "abc" and "abc1/2/3"
        for pair in pairs {
            if pair.0 == "abc" {
                assert_eq!(pair.1 .0, "abcabc".as_bytes().to_vec());
            } else if pair.0 == "abc1" {
                assert_eq!(pair.1 .0, "abcabc1".as_bytes().to_vec());
            } else if pair.0 == "abc2" {
                assert_eq!(pair.1 .0, "abcabc2".as_bytes().to_vec());
            } else if pair.0 == "abc3" {
                assert_eq!(pair.1 .0, "abcabc3".as_bytes().to_vec());
            } else {
                panic!("unexpected key");
            }
        }
    }
}
