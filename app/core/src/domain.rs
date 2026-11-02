//! Transport-neutral identities, lossless paths, and immutable object names.
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct WorkspaceId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct RepositoryId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct WorktreeId(pub String);

/// `display` is never used to reconstruct a path. Byte/wide encodings round-trip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "encoding", content = "value", rename_all = "snake_case")]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub enum NativePath {
    Utf8(String),
    UnixBytes(Vec<u8>),
    WindowsWide(Vec<u16>),
}
impl NativePath {
    pub fn from_path(path: &Path) -> Self {
        if let Some(text) = path.to_str() {
            return Self::Utf8(text.to_owned());
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            Self::UnixBytes(path.as_os_str().as_bytes().to_vec())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            Self::WindowsWide(path.as_os_str().encode_wide().collect())
        }
    }
    pub fn from_git(bytes: &[u8]) -> Result<Self> {
        if bytes.contains(&0) {
            return Err(invalid("NUL in Git path"));
        }
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok(Self::Utf8(text.to_owned()));
        }
        #[cfg(unix)]
        {
            Ok(Self::UnixBytes(bytes.to_vec()))
        }
        #[cfg(not(unix))]
        {
            Err(invalid(
                "Native Git path encoding unsupported on this platform",
            ))
        }
    }
    pub fn to_path(&self) -> Result<PathBuf> {
        match self {
            Self::Utf8(text) if !text.contains('\0') => Ok(PathBuf::from(text)),
            #[cfg(unix)]
            Self::UnixBytes(bytes) if !bytes.contains(&0) => {
                use std::os::unix::ffi::OsStringExt;
                Ok(OsString::from_vec(bytes.clone()).into())
            }
            #[cfg(windows)]
            Self::WindowsWide(wide) if !wide.contains(&0) => {
                use std::os::windows::ffi::OsStringExt;
                Ok(OsString::from_wide(wide).into())
            }
            _ => Err(invalid("Path encoding is invalid for this platform")),
        }
    }
    pub fn relative(&self) -> Result<PathBuf> {
        let path = self.to_path()?;
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(invalid("Expected a normalized repository-relative path"));
        }
        // Reject alternate separators/metadata spellings conservatively on all hosts.
        let bytes = path.as_os_str().as_encoded_bytes();
        if bytes.contains(&b'\\')
            || bytes
                .split(|b| *b == b'/')
                .any(|p| p.is_empty() || p == b"." || p == b".." || p.eq_ignore_ascii_case(b".git"))
        {
            return Err(invalid("Invalid relative path component"));
        }
        Ok(path)
    }
    pub fn display(&self) -> String {
        match self.to_path() {
            Ok(path) => format!("{}", path.display()).escape_default().to_string(),
            Err(_) => "<unsupported path encoding>".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ObjectFormat {
    Sha1,
    Sha256,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct ObjectId {
    pub format: ObjectFormat,
    pub hex: String,
}
impl ObjectId {
    pub fn parse(value: &str, format: &ObjectFormat) -> Result<Self> {
        let length = if *format == ObjectFormat::Sha1 {
            40
        } else {
            64
        };
        if value.len() != length
            || !value
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(Error::with_kind(
                ErrorKind::InvalidData,
                "Invalid object ID",
            ));
        }
        Ok(Self {
            format: format.clone(),
            hex: value.to_owned(),
        })
    }
    pub fn validate(&self) -> Result<()> {
        Self::parse(&self.hex, &self.format).map(|_| ())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Issue {
    pub kind: ErrorKind,
    pub message: String,
}
impl From<Error> for Issue {
    fn from(error: Error) -> Self {
        Self {
            kind: error.kind,
            message: error.to_string(),
        }
    }
}
pub(crate) fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidPath, message)
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
