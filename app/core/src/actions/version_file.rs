//! Descriptor-relative replacement of one reviewed regular version file.
//! Directory descriptors pin every ancestor; no symlink is followed during traversal.
use super::version::FileStamp;
use crate::{domain::NativePath, Error, ErrorKind, Result};
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        },
    },
    path::Path,
};

fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::ConcurrentChange, message)
}
fn name(bytes: &[u8]) -> Result<CString> {
    CString::new(bytes).map_err(|_| invalid("Invalid version filename"))
}
fn opened_at(parent: &File, name: &CString, flags: i32, mode: u32) -> Result<File> {
    // SAFETY: parent owns a live descriptor and name is NUL-terminated. A successful
    // openat returns a new owned descriptor, transferred exactly once to File.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            mode,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}
pub(crate) fn stamp(file: &File) -> Result<FileStamp> {
    let m = file.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o7000 != 0
        || m.len() > 1024
    {
        return Err(invalid("Version file must be an owned single-link regular file of at most 1024 bytes without special permission bits"));
    }
    Ok(FileStamp {
        device: m.dev(),
        inode: m.ino(),
        mode: m.mode(),
        length: m.len(),
        modified_seconds: m.mtime(),
        modified_nanos: m.mtime_nsec(),
        changed_seconds: m.ctime(),
        changed_nanos: m.ctime_nsec(),
    })
}
pub(crate) struct Target {
    pub root: File,
    pub parent: File,
    leaf: CString,
}
impl Target {
    pub fn open(root: &Path, path: &NativePath) -> Result<Self> {
        let relative = path.relative()?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
        let root = options.open(root)?;
        let mut parent = root.try_clone()?;
        let parts: Vec<_> = relative.components().collect();
        for part in &parts[..parts.len() - 1] {
            parent = opened_at(
                &parent,
                &name(part.as_os_str().as_bytes())?,
                libc::O_RDONLY | libc::O_DIRECTORY,
                0,
            )?;
            // No action may cross into an independently owned repository/worktree.
            let dotgit = name(b".git")?;
            let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
            // SAFETY: stat points to writable storage; AT_SYMLINK_NOFOLLOW inspects
            // the marker itself without dereferencing it.
            let rc = unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    dotgit.as_ptr(),
                    stat.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if rc == 0 {
                return Err(invalid(
                    "Version target crosses a nested repository boundary",
                ));
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(error.into());
            }
        }
        Ok(Self {
            root,
            parent,
            leaf: name(
                parts
                    .last()
                    .expect("validated relative path")
                    .as_os_str()
                    .as_bytes(),
            )?,
        })
    }
    pub fn identities(&self) -> Result<(u64, u64, u64, u64)> {
        let root = self.root.metadata()?;
        let parent = self.parent.metadata()?;
        Ok((root.dev(), root.ino(), parent.dev(), parent.ino()))
    }
    pub fn read(&self) -> Result<(FileStamp, Vec<u8>)> {
        let mut file = opened_at(&self.parent, &self.leaf, libc::O_RDONLY, 0)?;
        let before = stamp(&file)?;
        let mut bytes = Vec::new();
        std::io::Read::by_ref(&mut file)
            .take(1025)
            .read_to_end(&mut bytes)?;
        if before != stamp(&file)? || bytes.len() > 1024 {
            return Err(invalid("Version file changed during inspection"));
        }
        Ok((before, bytes))
    }
    pub fn candidate(&self, artifact: &File, bytes: &[u8], mode: u32) -> Result<FileStamp> {
        if artifact.metadata()?.dev() != self.parent.metadata()?.dev() {
            return Err(invalid(
                "Version recovery storage must share the target filesystem",
            ));
        }
        let mut file = opened_at(
            artifact,
            &name(b"version.candidate")?,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        file.write_all(bytes)?;
        file.set_permissions(std::fs::Permissions::from_mode(mode & 0o777))?;
        sync_file(&file)?;
        artifact.sync_all()?;
        stamp(&file)
    }
    pub fn replace(
        &self,
        artifact: &File,
        candidate: &FileStamp,
        expected: &FileStamp,
        before: &[u8],
    ) -> Result<()> {
        let (current, bytes) = self.read()?;
        if &current != expected || bytes != before {
            return Err(invalid(
                "Version file changed after review; no exchange performed",
            ));
        }
        let file = opened_at(artifact, &name(b"version.candidate")?, libc::O_RDONLY, 0)?;
        if &stamp(&file)? != candidate {
            return Err(invalid("Version candidate identity changed"));
        }
        self.exchange_checked(artifact, expected, before)
    }
    fn exchange_checked(&self, artifact: &File, expected: &FileStamp, before: &[u8]) -> Result<()> {
        let candidate = name(b"version.candidate")?;
        atomic_swap(artifact, &candidate, &self.parent, &self.leaf)?;
        self.parent.sync_all()?;
        artifact.sync_all()?;
        sync_file(&opened_at(artifact, &candidate, libc::O_RDONLY, 0)?)?;
        // A writer may race the last precheck. Exchange retains that exact old
        // object in private storage rather than destroying its unexpected bytes.
        let (old, bytes) = read_artifact(artifact)?;
        if !same_previous_file(&old, expected) || bytes != before {
            return Err(Error::with_kind(ErrorKind::UncertainOutcome,"Concurrent version edit retained in private recovery storage; inspect without retry"));
        }
        Ok(())
    }
}
fn sync_file(file: &File) -> Result<()> {
    file.sync_all()?;
    #[cfg(target_os = "macos")]
    // SAFETY: file owns a live descriptor; F_FULLFSYNC takes no extra argument.
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
pub(crate) fn private_artifact(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path)?;
    let m = file.metadata()?;
    if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
        return Err(invalid("Unsafe private version recovery directory"));
    }
    Ok(file)
}
pub(crate) fn read_artifact(artifact: &File) -> Result<(FileStamp, Vec<u8>)> {
    let mut file = opened_at(artifact, &name(b"version.candidate")?, libc::O_RDONLY, 0)?;
    let before = stamp(&file)?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(1025)
        .read_to_end(&mut bytes)?;
    if before != stamp(&file)? || bytes.len() > 1024 {
        return Err(invalid("Version recovery file changed during inspection"));
    }
    Ok((before, bytes))
}
pub(crate) fn same_previous_file(current: &FileStamp, expected: &FileStamp) -> bool {
    // Atomic exchange changes ctime. Inode, mtime, mode, length and exact bytes
    // establish the retained original; ctime remains in normal review stamps.
    current.device == expected.device
        && current.inode == expected.inode
        && current.mode == expected.mode
        && current.length == expected.length
        && current.modified_seconds == expected.modified_seconds
        && current.modified_nanos == expected.modified_nanos
}
#[cfg(target_os = "macos")]
fn atomic_swap(from: &File, from_name: &CString, to: &File, to_name: &CString) -> Result<()> {
    // macOS sys/stdio.h; this SDK flag is absent from libc 0.2.191.
    const RENAME_NOFOLLOW_ANY: libc::c_uint = 0x00000010;
    // SAFETY: both descriptors and NUL-terminated names are live and owned. This
    // operation exchanges names on one filesystem; no overwrite fallback exists.
    if unsafe {
        libc::renameatx_np(
            from.as_raw_fd(),
            from_name.as_ptr(),
            to.as_raw_fd(),
            to_name.as_ptr(),
            libc::RENAME_SWAP | RENAME_NOFOLLOW_ANY,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn atomic_swap(_from: &File, _from_name: &CString, _to: &File, _to_name: &CString) -> Result<()> {
    Err(Error::with_kind(
        ErrorKind::UnsupportedConfiguration,
        "Version Action execution currently requires tested macOS atomic exchange",
    ))
}
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn racing_external_edit_is_retained_by_exchange_instead_of_deleted() {
        let dir = std::env::temp_dir().join(format!(
            "gitfleet-version-exchange-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir(&dir).unwrap();
        let recovery = dir.join("private");
        std::fs::create_dir(&recovery).unwrap();
        std::fs::set_permissions(&recovery, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(dir.join("version"), b"1.4.2\n").unwrap();
        let target = Target::open(&dir, &NativePath::Utf8("version".into())).unwrap();
        let (before, bytes) = target.read().unwrap();
        let artifact = private_artifact(&recovery).unwrap();
        target
            .candidate(&artifact, b"1.4.3\n", before.mode)
            .unwrap();
        // Deliberately write after the caller's precheck and before the exchange.
        std::fs::write(dir.join("version"), b"concurrent external edit\n").unwrap();
        let error = target
            .exchange_checked(&artifact, &before, &bytes)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::UncertainOutcome);
        assert_eq!(std::fs::read(dir.join("version")).unwrap(), b"1.4.3\n");
        assert_eq!(
            read_artifact(&artifact).unwrap().1,
            b"concurrent external edit\n"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
