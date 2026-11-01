use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Error(pub String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Self(e.to_string()) }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self { Self(e.to_string()) }
}

pub type Result<T> = std::result::Result<T, Error>;
