use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::TimeDelta;
use tokio::sync::Mutex;

use super::client::CacheStorage;

pub struct LocalStorage {
    map: Arc<Mutex<HashMap<String, String>>>,
    expiration_map: Arc<Mutex<HashMap<String, TimeDelta>>>,
}

#[async_trait]
impl CacheStorage for LocalStorage {
    async fn connect() -> Self {
        let map = Arc::new(Mutex::new(HashMap::new()));
        let expiration_map = Arc::new(Mutex::new(HashMap::new()));

        Self {
            map,
            expiration_map,
        }
    }

    async fn set(&self, key: String, value: String) -> Result<(), ()> {
        let mut map = self.map.lock().await;
        map.insert(key.to_string(), value);
        Ok(())
    }

    async fn expire(&self, key: String, time: TimeDelta) -> Result<(), ()> {
        let mut map = self.expiration_map.lock().await;
        map.insert(key.to_string(), time);
        Ok(())
    }

    async fn get<'s>(&self, key: &'s str) -> Option<String> {
        let map = self.map.lock().await;
        let v = map.get(key);

        if let Some(value) = v {
            Some(value.clone())
        } else {
            None
        }
    }

    async fn delete<'s>(&self, key: &'s str) {
        let mut map = self.map.lock().await;
        map.remove(key);
    }
}
