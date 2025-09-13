use super::{
    client::{Database, DatabaseUser},
    data::{Credentials, User},
    error::Error,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, migrate::MigrateError, sqlite::SqlitePool};
use uuid::Uuid;

pub struct Sqlite {
    pool: SqlitePool,
}

#[async_trait]
impl Database for Sqlite {
    async fn connect() -> Self {
        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("Failed to connect sqlite database.");

        Sqlite { pool }
    }

    async fn migrate(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations_sqlite").run(&self.pool).await
    }
}

#[async_trait]
impl DatabaseUser for Sqlite {
    async fn create_user(&self, user: User, creds: Credentials) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let result = sqlx::query(r#"
        INSERT INTO Users (id, name) 
            VALUES (?,?);
        INSERT INTO UserCredentials (id, user_id, email, username, password_hash, password_salt, created_at, updated_at)
            VALUES (?,?,?,?,?,?,?,?);
        "#)
            .bind(user.id.to_string())
            .bind(user.name)

            .bind(creds.id.to_string())
            .bind(creds.user_id.to_string())
            .bind(creds.email)
            .bind(creds.username)
            .bind(creds.password_hash)
            .bind(creds.password_salt)
            .bind(creds.created_at.to_rfc3339())
            .bind(creds.updated_at.to_rfc3339())
            .execute(&mut conn)
            .await
            .map_err(|err| Error::DB(err))?;

        if result.rows_affected() == 0 {
            return Err(Error::DbNoEffect);
        }

        Ok(())
    }

    async fn get_user_by_email<'s>(&self, email: &'s str) -> Result<User, Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let row = sqlx::query(
            r#"
        SELECT id,name FROM Users
        WHERE id = (SELECT user_id FROM UserCredentials WHERE email = ?);
        "#,
        )
        .bind(email)
        .fetch_one(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        let id_str: String = row.try_get("id").map_err(|err| Error::DB(err))?;
        let name: String = row.try_get("name").map_err(|err| Error::DB(err))?;

        let id = Uuid::parse_str(&id_str).map_err(|err| Error::Uuid(err))?;

        Ok(User { id, name })
    }
    async fn get_user_by_username<'s>(&self, username: &'s str) -> Result<User, Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let row = sqlx::query(
            r#"
        SELECT id, name FROM Users
        WHERE id = (SELECT user_id FROM UserCredentials WHERE username = ?)
        "#,
        )
        .bind(username)
        .fetch_one(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        let id_str: String = row.try_get("id").map_err(|err| Error::DB(err))?;
        let name: String = row.try_get("name").map_err(|err| Error::DB(err))?;

        let id = Uuid::parse_str(&id_str).map_err(|err| Error::Uuid(err))?;

        Ok(User { id, name })
    }

    async fn get_user_creds<'s>(&self, user_id: &'s str) -> Result<Credentials, Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let row = sqlx::query(
            r#"
        SELECT id, user_id, email, username, password_hash, password_salt, created_at, updated_at FROM UserCredentials;
        WHERE user_id = ?;
        "#,
        )
        .bind(user_id)
        .fetch_one(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        let id_str: String = row.try_get("id").map_err(|err| Error::DB(err))?;
        let user_id_str: String = row.try_get("user_id").map_err(|err| Error::DB(err))?;
        let email: String = row.try_get("email").map_err(|err| Error::DB(err))?;
        let username: String = row.try_get("username").map_err(|err| Error::DB(err))?;
        let password_hash: String = row.try_get("password_hash").map_err(|err| Error::DB(err))?;
        let password_salt: String = row.try_get("password_salt").map_err(|err| Error::DB(err))?;
        let created_at_str: String = row.try_get("created_at").map_err(|err| Error::DB(err))?;
        let updated_at_str: String = row.try_get("updated_at").map_err(|err| Error::DB(err))?;

        let id = Uuid::parse_str(&id_str).map_err(|err| Error::Uuid(err))?;
        let user_id = Uuid::parse_str(&user_id_str).map_err(|err| Error::Uuid(err))?;

        let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|err| Error::Chrono(err))?
            .to_utc();

        let updated_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|err| Error::Chrono(err))?
            .to_utc();

        Ok(Credentials {
            id,
            user_id,
            email,
            username,
            password_hash,
            password_salt,
            created_at,
            updated_at,
        })
    }

    async fn update_user_email(&self, id: Uuid, email: String) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let result = sqlx::query(
            r#"
        UPDATE UserCredentials
            SET email = ?
        WHERE user_id = ?;
        "#,
        )
        .bind(email)
        .bind(id.to_string())
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        if result.rows_affected() == 0 {
            return Err(Error::DbNoEffect);
        }

        Ok(())
    }

    async fn update_user_username(&self, id: Uuid, username: String) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let result = sqlx::query(
            r#"
        UPDATE UserCredentials
            set username = ?
        WHERE user_id = ?;
        "#,
        )
        .bind(username)
        .bind(id.to_string())
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        if result.rows_affected() == 0 {
            return Err(Error::DbNoEffect);
        }

        Ok(())
    }

    async fn update_user_password_hash(
        &self,
        id: Uuid,
        password_hash: String,
    ) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let result = sqlx::query(
            r#"
        UPDATE UserCredentials
            SET password_hash = ?
        WHERE user_id = ?;
        "#,
        )
        .bind(password_hash)
        .bind(id.to_string())
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        if result.rows_affected() == 0 {
            return Err(Error::DbNoEffect);
        }

        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await.map_err(|err| Error::DB(err))?;

        let result = sqlx::query(
            r#"
        DELETE FROM Users WHERE id = ?;
        DELETE FROM UserCredentials WHERE user_id = ?;
        "#,
        )
        .bind(id.to_string())
        .bind(id.to_string())
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DB(err))?;

        if result.rows_affected() == 0 {
            return Err(Error::DbNoEffect);
        }

        Ok(())
    }
}
