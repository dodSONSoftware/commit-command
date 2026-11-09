//! Typed deterministic Actions. Preview functions never grant execution authority.
pub mod semver;
#[cfg(feature = "application-services")]
pub mod version;
#[cfg(feature = "application-services")]
pub(crate) mod version_file;
use crate::{
    domain::NativePath,
    inspection::{self, ReadOptions},
    Error, ErrorKind, Result,
};
use std::path::Path;

/// Inspect only a regular file owned by this repository, never a nested checkout.
pub fn preview_version_file(
    repo: &Path,
    path: &NativePath,
    increment: semver::Increment,
    prerelease: semver::PrereleasePolicy,
    options: &ReadOptions,
) -> Result<semver::VersionPreview> {
    inspection::read(repo, options, |session| {
        let target = session.safe_path(path)?;
        let relative = path.relative()?;
        let mut ancestor = session.root.clone();
        let components: Vec<_> = relative.components().collect();
        for component in &components[..components.len() - 1] {
            ancestor.push(component.as_os_str());
            match std::fs::symlink_metadata(ancestor.join(".git")) {
                Ok(_) => {
                    return Err(Error::with_kind(
                        ErrorKind::InvalidPath,
                        "Select the owning repository for a nested version file",
                    ))
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        let meta = std::fs::symlink_metadata(&target)?;
        if !meta.is_file() || meta.is_symlink() {
            return Err(Error::with_kind(
                ErrorKind::InvalidPath,
                "Version target must be a regular file, never a symlink",
            ));
        }
        semver::preview(
            &inspection::file_bytes(&target, 1024)?,
            increment,
            prerelease,
        )
    })
}
