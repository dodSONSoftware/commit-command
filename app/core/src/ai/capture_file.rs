//! Read-only descriptor traversal for AI capture; never follows links or crosses repositories.
use crate::{
    domain::{digest, NativePath},
    Error, ErrorKind, Result,
};
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    io::Read,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::Path,
};
fn refused() -> Error {
    Error::with_kind(
        ErrorKind::InvalidPath,
        "AI source must be an owned single-link regular file inside this worktree",
    )
}
fn open_at(parent: &File, name: &CString, directory: bool) -> std::io::Result<File> {
    let flags = libc::O_RDONLY
        | libc::O_NOFOLLOW
        | libc::O_NONBLOCK
        | libc::O_CLOEXEC
        | if directory { libc::O_DIRECTORY } else { 0 };
    // SAFETY: live owned directory descriptor and a valid NUL-terminated name;
    // successful openat transfers exactly one new descriptor to File.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn stamp(file: &File) -> Result<String> {
    let m = file.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o7000 != 0
    {
        return Err(refused());
    }
    Ok(format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        m.dev(),
        m.ino(),
        m.mode(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec()
    ))
}
pub(super) fn read(
    root: &Path,
    path: &NativePath,
    limit: usize,
    allow_missing: bool,
) -> Result<(Vec<u8>, String)> {
    let relative = path.relative()?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut parent = options.open(root)?;
    let parts = relative.components().collect::<Vec<_>>();
    let names = parts
        .iter()
        .map(|p| CString::new(p.as_os_str().as_bytes()).map_err(|_| refused()))
        .collect::<Result<Vec<_>>>()?;
    for part in &names[..names.len() - 1] {
        parent = open_at(&parent, part, true)?;
        let dotgit = CString::new(".git").map_err(|_| refused())?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: metadata points to writable storage, descriptor and C name live.
        if unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                dotgit.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0
            || std::io::Error::last_os_error().kind() != std::io::ErrorKind::NotFound
        {
            return Err(refused());
        }
    }
    let mut file = match open_at(&parent, names.last().ok_or_else(refused)?, false) {
        Ok(file) => file,
        Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => {
            let p = parent.metadata()?;
            return Ok((Vec::new(), format!("absent:{}:{}", p.dev(), p.ino())));
        }
        Err(error) => return Err(error.into()),
    };
    let before = stamp(&file)?;
    if file.metadata()?.len() > limit as u64 {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "AI source file exceeds capture bound",
        ));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit || before != stamp(&file)? {
        return Err(Error::with_kind(
            ErrorKind::ConcurrentChange,
            "AI source changed during capture",
        ));
    }
    let fingerprint = digest(&[before.as_bytes(), &bytes].concat());
    Ok((bytes, fingerprint))
}
