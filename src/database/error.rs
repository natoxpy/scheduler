use std::fmt;

#[derive(Debug)]
pub enum Error {
    DB(sqlx::Error),
    DbNoEffect,
    Uuid(uuid::Error),
    Chrono(chrono::ParseError),
    Argon2Hasher(argon2::password_hash::Error),
    Message(String),
    Cache,
}

impl Into<poem::error::Error> for Error {
    fn into(self) -> poem::error::Error {
        todo!()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DB(error) => write!(f, "{}", error),
            Error::DbNoEffect => write!(f, "No Effect Error"),
            Error::Uuid(error) => write!(f, "{}", error),
            Error::Chrono(parse_error) => write!(f, "{}", parse_error),
            Error::Argon2Hasher(error) => write!(f, "{}", error),
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Cache => write!(f, "Cache error"),
        }
    }
}
