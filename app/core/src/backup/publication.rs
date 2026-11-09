//! Private staging and no-clobber publication, anchored to a directory descriptor.
use super::archive::{self, Manifest, Source, ValidatedArchive};
use crate::{inspection::Cancellation, Error, ErrorKind, Result};
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path},
};
use uuid::Uuid;

fn refused(message: &str) -> Error {
    Error::with_kind(ErrorKind::Storage, message)
}
fn component(name: &str) -> Result<CString> {
    if name.is_empty()
        || name.len() > 255
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.chars().any(char::is_control)
    {
        return Err(refused("Backup destination must be a single file name"));
    }
    CString::new(name).map_err(|_| refused("Invalid backup destination name"))
}
fn open_child(
    parent: &File,
    name: &std::ffi::CStr,
    flags: i32,
    mode: libc::mode_t,
) -> Result<File> {
    // Every returned descriptor is immediately owned, and names have no separators.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            libc::c_uint::from(mode),
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

/// The final directory must already exist, be owned by this user and private.
/// Ancestors are traversed without following symlinks; callers choose an absolute
/// directory outside managed repositories under the application storage policy.
pub struct Destination {
    directory: File,
}
impl Destination {
    pub fn open(path: &Path) -> Result<Self> {
        if !path.is_absolute() {
            return Err(refused("Backup directory must be absolute"));
        }
        let mut directory = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open("/")?;
        for part in path.components() {
            match part {
                Component::RootDir => {}
                Component::Normal(name) => {
                    use std::os::unix::ffi::OsStrExt;
                    let name = CString::new(name.as_bytes())
                        .map_err(|_| refused("Invalid backup directory"))?;
                    directory =
                        open_child(&directory, &name, libc::O_RDONLY | libc::O_DIRECTORY, 0)?;
                }
                _ => {
                    return Err(refused(
                        "Backup directory contains unsupported path components",
                    ))
                }
            }
        }
        let metadata = directory.metadata()?;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(refused(
                "Backup directory must be owned by this user with mode 0700",
            ));
        }
        Ok(Self { directory })
    }

    /// Publishes a new name only. Existing backups are never overwritten.
    /// Success follows file sync, independent container verification, atomic link,
    /// temporary-name removal and directory sync. An error after link publication
    /// may leave a valid new archive; callers must not report success or rotate.
    pub fn publish(
        &self,
        name: &str,
        manifest: &Manifest,
        sources: &mut [Source<'_>],
        cancel: &Cancellation,
    ) -> Result<String> {
        let metadata = self.directory.metadata()?;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(refused("Backup directory is no longer private"));
        }
        let final_name = component(name)?;
        if name.starts_with(".backup-") {
            return Err(refused("Backup name uses reserved staging prefix"));
        }
        if cancel.is_cancelled() {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Backup cancelled before staging",
            ));
        }
        let temporary = component(&format!(".backup-{}.tmp", Uuid::new_v4()))?;
        let file = open_child(
            &self.directory,
            &temporary,
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        let staged = Staged {
            directory: &self.directory,
            name: temporary,
            file,
        };
        archive::write_archive(staged.file.try_clone()?, manifest, sources, cancel)?;
        staged.file.sync_all()?;
        #[cfg(target_os = "macos")]
        if unsafe { libc::fcntl(staged.file.as_raw_fd(), libc::F_FULLFSYNC) } < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        use std::io::{Seek, SeekFrom};
        let mut reader = staged.file.try_clone()?;
        reader.seek(SeekFrom::Start(0))?;
        let checked = ValidatedArchive::inspect(reader, cancel)?;
        let fingerprint = checked.manifest_fingerprint().to_owned();
        drop(checked);
        if cancel.is_cancelled() {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Backup cancelled before publication",
            ));
        }
        staged.verify_name()?;
        // linkat without AT_SYMLINK_FOLLOW is atomic and refuses an existing name.
        // Both names are relative to the same held directory, hence same filesystem.
        if unsafe {
            libc::linkat(
                self.directory.as_raw_fd(),
                staged.name.as_ptr(),
                self.directory.as_raw_fd(),
                final_name.as_ptr(),
                0,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        staged.remove()?;
        self.directory.sync_all()?;
        Ok(fingerprint)
    }
}

struct Staged<'a> {
    directory: &'a File,
    name: CString,
    file: File,
}
impl Staged<'_> {
    fn verify_name(&self) -> Result<()> {
        let named = open_child(
            self.directory,
            &self.name,
            libc::O_RDONLY | libc::O_NONBLOCK,
            0,
        )?;
        let actual = named.metadata()?;
        let expected = self.file.metadata()?;
        if !actual.is_file()
            || actual.dev() != expected.dev()
            || actual.ino() != expected.ino()
            || actual.nlink() != 1
            || actual.uid() != unsafe { libc::geteuid() }
            || actual.mode() & 0o077 != 0
        {
            return Err(refused("Backup staging identity changed"));
        }
        Ok(())
    }
    fn remove(&self) -> Result<()> {
        // After publication the expected inode has two names, so compare identity
        // independently of the pre-publication single-link check.
        let named = open_child(
            self.directory,
            &self.name,
            libc::O_RDONLY | libc::O_NONBLOCK,
            0,
        )?;
        let actual = named.metadata()?;
        let expected = self.file.metadata()?;
        if actual.dev() != expected.dev() || actual.ino() != expected.ino() {
            return Err(refused("Backup temporary identity changed"));
        }
        if unsafe { libc::unlinkat(self.directory.as_raw_fd(), self.name.as_ptr(), 0) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
}
impl Drop for Staged<'_> {
    fn drop(&mut self) {
        // Best-effort cleanup only of this still-identical private temporary file.
        // Failure leaves an inert .tmp; no startup or reader treats it as a backup.
        let _ = self.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::super::archive::{Component as ArchiveComponent, ComponentType, Kind};
    use super::*;
    use std::{
        fs,
        io::Cursor,
        os::unix::fs::{symlink, PermissionsExt},
    };
    fn fixture() -> (std::path::PathBuf, Manifest) {
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("gitfleet-backup-{}", Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let manifest = Manifest {
            version: 1,
            application_version: "0.1.1".into(),
            created_at_ms: 1,
            kind: Kind::AiConfiguration,
            database_schema: None,
            settings_revision: Some("fixture".into()),
            includes_history: false,
            recovery_policy: "not_included".into(),
            components: vec![ArchiveComponent {
                component_type: ComponentType::RegularFile,
                path: "ai/settings.json".into(),
                bytes: 1,
                sha256: crate::domain::digest(b"x"),
            }],
        };
        (root, manifest)
    }
    fn publish(
        destination: &Destination,
        name: &str,
        manifest: &Manifest,
        bytes: &[u8],
        cancel: &Cancellation,
    ) -> Result<String> {
        destination.publish(
            name,
            manifest,
            &mut [Source {
                path: "ai/settings.json",
                reader: &mut Cursor::new(bytes),
            }],
            cancel,
        )
    }
    #[test]
    fn publishes_verified_private_archive_and_never_clobbers() {
        let (root, manifest) = fixture();
        let destination = Destination::open(&root).unwrap();
        let fingerprint = publish(
            &destination,
            "manual.zip",
            &manifest,
            b"x",
            &Cancellation::default(),
        )
        .unwrap();
        let original = fs::read(root.join("manual.zip")).unwrap();
        let checked =
            ValidatedArchive::inspect(Cursor::new(&original), &Cancellation::default()).unwrap();
        assert_eq!(checked.manifest_fingerprint(), fingerprint);
        assert_eq!(
            fs::metadata(root.join("manual.zip")).unwrap().mode() & 0o777,
            0o600
        );
        assert_eq!(fs::metadata(root.join("manual.zip")).unwrap().nlink(), 1);
        assert!(publish(
            &destination,
            "manual.zip",
            &manifest,
            b"x",
            &Cancellation::default()
        )
        .is_err());
        assert_eq!(fs::read(root.join("manual.zip")).unwrap(), original);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn refused_paths_links_and_failed_sources_preserve_destination() {
        let (root, manifest) = fixture();
        let destination = Destination::open(&root).unwrap();
        fs::write(root.join("keep"), b"original").unwrap();
        symlink("keep", root.join("link.zip")).unwrap();
        for name in [
            "../escape",
            "dir/file",
            "",
            "link.zip",
            ".backup-reserved.zip",
        ] {
            assert!(publish(
                &destination,
                name,
                &manifest,
                b"x",
                &Cancellation::default()
            )
            .is_err());
        }
        assert!(publish(
            &destination,
            "bad.zip",
            &manifest,
            b"wrong",
            &Cancellation::default()
        )
        .is_err());
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(publish(&destination, "cancelled.zip", &manifest, b"x", &cancel).is_err());
        assert_eq!(fs::read(root.join("keep")).unwrap(), b"original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        symlink(&root, root.join("alias")).unwrap();
        assert!(Destination::open(&root.join("alias")).is_err());
        assert!(Destination::open(Path::new("relative")).is_err());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(Destination::open(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cancellation_and_io_failure_after_staging_leave_no_published_backup() {
        struct CancelReader<'a>(&'a Cancellation);
        impl std::io::Read for CancelReader<'_> {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                buffer[0] = b'x';
                self.0.cancel();
                Ok(1)
            }
        }
        struct FailedReader;
        impl std::io::Read for FailedReader {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected source read failure"))
            }
        }
        let (root, manifest) = fixture();
        let destination = Destination::open(&root).unwrap();
        fs::write(root.join("previous.zip"), b"previous bytes").unwrap();
        let cancel = Cancellation::default();
        let result = destination.publish(
            "cancelled.zip",
            &manifest,
            &mut [Source {
                path: "ai/settings.json",
                reader: &mut CancelReader(&cancel),
            }],
            &cancel,
        );
        assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
        assert!(destination
            .publish(
                "failed.zip",
                &manifest,
                &mut [Source {
                    path: "ai/settings.json",
                    reader: &mut FailedReader
                }],
                &Cancellation::default()
            )
            .is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert_eq!(
            fs::read(root.join("previous.zip")).unwrap(),
            b"previous bytes"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
