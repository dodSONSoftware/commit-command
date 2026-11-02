use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    InvalidPath,
    UnsupportedConfiguration,
    InspectionFailed,
    HookRejected,
    Io,
    InvalidData,
    Experimental,
    InvalidRequest,
    UnsupportedGitVersion,
    Unavailable,
    ConcurrentChange,
    StaleCursor,
    MissingObject,
    Timeout,
    Cancelled,
    OutputLimit,
    Busy,
    Storage,
    NewerSchema,
    UntrustedRepository,
    UnsupportedRepositoryState,
    StaleReview,
    SigningFailed,
    NeedsManualReconciliation,
    UncertainOutcome,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self::with_kind(ErrorKind::Experimental, message)
    }

    pub fn with_kind(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::with_kind(ErrorKind::Io, e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::with_kind(ErrorKind::InvalidData, e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::with_kind(ErrorKind::Storage, "SQLite operation failed")
    }
}
