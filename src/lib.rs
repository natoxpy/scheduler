use cache::client::CacheStorage;
use chrono::TimeDelta;
use database::client::Database;
use uuid::Uuid;

pub mod cache;
pub mod database;
pub mod schedule;
pub mod server;
pub mod storage;
pub mod task;

pub struct AppState {
    pub database: Box<dyn Database>,
    pub cache: Box<dyn CacheStorage>,
}

impl AppState {
    pub async fn connect<DB: Database + 'static, CS: CacheStorage + 'static>() -> Self {
        Self {
            database: Box::new(DB::connect().await),
            cache: Box::new(CS::connect().await),
        }
    }
}

impl AppState {
    pub async fn authenticate(
        &self,
        username: &Option<String>,
        email: &Option<String>,
        password: &String,
    ) -> Result<String, crate::database::error::Error> {
        let user = if let Some(username) = username {
            self.database.get_user_by_email(username).await?
        } else if let Some(email) = email {
            self.database.get_user_by_email(email).await?
        } else {
            return Err(crate::database::error::Error::Message(String::from(
                "Missing email or username!",
            )));
        };

        println!("{:?}", user);

        let creds = self.database.get_user_creds(&user.id.to_string()).await?;

        if !creds.check_password(password.as_bytes())? {
            return Err(crate::database::error::Error::Message(String::from(
                "Password mismatch!",
            )));
        }

        let uuid = Uuid::new_v4().to_string();
        let key = format!("session:{}", uuid);

        self.cache
            .set(key.clone(), user.id.to_string())
            .await
            .map_err(|_| crate::database::error::Error::Cache)?;

        self.cache
            .expire(key, TimeDelta::hours(24))
            .await
            .map_err(|_| crate::database::error::Error::Cache)?;

        Ok(uuid)
    }
}
