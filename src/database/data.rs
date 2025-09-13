use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

impl User {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
        }
    }
}

#[derive(Debug)]
pub struct CredentialsNoPassword {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CredentialsNoPassword {
    pub fn add_password_and_salt<'a>(
        self,
        password: impl Into<&'a [u8]>,
    ) -> Result<Credentials, crate::database::error::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.into(), &salt)
            .map_err(|err| crate::database::error::Error::Argon2Hasher(err))?
            .to_string();

        let password_salt = salt.to_string();

        Ok(Credentials {
            id: self.id,
            user_id: self.user_id,
            email: self.email,
            username: self.username,
            password_hash,
            password_salt,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug)]
pub struct Credentials {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Credentials {
    pub fn new(
        user_id: Uuid,
        email: impl Into<String>,
        username: impl Into<String>,
    ) -> CredentialsNoPassword {
        CredentialsNoPassword {
            id: Uuid::new_v4(),
            user_id: user_id.into(),
            email: email.into(),
            username: username.into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn check_password<'s>(
        &self,
        password: impl Into<&'s [u8]>,
    ) -> Result<bool, crate::database::error::Error> {
        let salt = SaltString::from_b64(&self.password_salt)
            .map_err(|err| crate::database::error::Error::Argon2Hasher(err))?;
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.into(), &salt)
            .map_err(|err| crate::database::error::Error::Argon2Hasher(err))?
            .to_string();

        Ok(password_hash == self.password_hash)
    }
}
