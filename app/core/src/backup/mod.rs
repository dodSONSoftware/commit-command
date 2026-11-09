//! Backup building blocks. No parsing operation extracts files or activates state.

pub const MAX_COMPONENT_BYTES: u64 = 256 * 1024 * 1024;

#[cfg(feature = "application-services")]
pub mod archive;
#[cfg(all(unix, feature = "application-services"))]
pub mod publication;
pub mod snapshot;
