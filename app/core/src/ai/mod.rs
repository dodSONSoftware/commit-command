//! Optional AI services. Configuration and generated text confer no write authority.
pub mod candidates;
pub mod capture;
#[cfg(unix)]
mod capture_file;
pub mod change;
pub mod config;
pub mod context;
pub mod credentials;
pub(crate) mod editor;
pub(crate) mod jobs;
#[cfg(unix)]
pub mod portability;
pub mod provider;
pub mod store;
pub mod template;
#[cfg(test)]
mod tests;

#[cfg(not(unix))]
mod capture_file {
    pub(super) fn read(
        _: &std::path::Path,
        _: &crate::domain::NativePath,
        _: usize,
        _: bool,
    ) -> crate::Result<(Vec<u8>, String)> {
        Err(crate::Error::with_kind(
            crate::ErrorKind::Unavailable,
            "Confined AI source capture is unavailable on this platform",
        ))
    }
}
