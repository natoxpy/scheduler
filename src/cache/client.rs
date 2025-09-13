use async_trait::async_trait;
use chrono::TimeDelta;

#[async_trait]
pub trait CacheStorage: Send + Sync {
    async fn set(&self, key: String, value: String) -> Result<(), ()>;
    async fn expire(&self, key: String, time: TimeDelta) -> Result<(), ()>;
    async fn get<'s>(&self, key: &'s str) -> Option<String>;
    async fn delete<'s>(&self, key: &'s str);

    async fn connect() -> Self
    where
        Self: Sized;
}
