use async_trait::async_trait;
use sqlx::migrate::MigrateError;
use uuid::Uuid;

use super::data::{Credentials, User};
use super::error::Error;

#[async_trait]
pub trait DatabaseUser {
    ///
    /// # Example
    ///
    /// ```rust
    /// // TODO: Add example
    /// ```
    ///
    async fn create_user(&self, user: User, creds: Credentials) -> Result<(), Error>;

    async fn get_user_by_email<'s>(&self, email: &'s str) -> Result<User, Error>;
    async fn get_user_by_username<'s>(&self, username: &'s str) -> Result<User, Error>;
    async fn get_user_creds<'s>(&self, user_id: &'s str) -> Result<Credentials, Error>;

    async fn update_user_email(&self, user_id: Uuid, email: String) -> Result<(), Error>;
    async fn update_user_username(&self, user_id: Uuid, username: String) -> Result<(), Error>;
    async fn update_user_password_hash(
        &self,
        user_id: Uuid,
        password_hash: String,
    ) -> Result<(), Error>;

    async fn delete_user(&self, user_id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait Database: Send + Sync + DatabaseUser {
    async fn connect() -> Self
    where
        Self: Sized;

    async fn migrate(&self) -> Result<(), MigrateError>;
}
