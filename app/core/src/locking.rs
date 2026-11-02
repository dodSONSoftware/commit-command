//! Explicit lock ownership: descriptor close alone can leave a lock alive in a
//! briefly inherited/duplicated descriptor while another thread spawns a child.
use crate::{Error, ErrorKind, Result};
use std::fs::{File, TryLockError};
pub(crate) struct ExclusiveLock {
    file: File,
}
impl ExclusiveLock {
    pub(crate) fn acquire(file: File, message: &str) -> Result<Self> {
        file.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => Error::with_kind(ErrorKind::Busy, message),
            TryLockError::Error(error) => error.into(),
        })?;
        Ok(Self { file })
    }
}
impl Drop for ExclusiveLock {
    fn drop(&mut self) {
        // LOCK_UN releases this ownership even if a child still holds a duplicate.
        // Closing the owned descriptor remains the fallback if unlock itself fails.
        if let Err(error) = self.file.unlock() {
            eprintln!(
                "GitFleet lock release failed: {:?}; closing owned descriptor",
                error.kind()
            );
        }
    }
}
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn explicit_release_does_not_wait_for_duplicated_descriptors() {
        let path = std::env::temp_dir().join(format!("gitfleet-lock-{}", uuid::Uuid::new_v4()));
        let open = || {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
                .unwrap()
        };
        let raw = open();
        raw.try_lock().unwrap();
        let inherited = raw.try_clone().unwrap();
        drop(raw);
        assert_eq!(
            ExclusiveLock::acquire(open(), "close alone retains ownership")
                .err()
                .unwrap()
                .kind,
            ErrorKind::Busy
        );
        // Explicitly end the negative control's ownership as well: another unit
        // test may be spawning a child while this descriptor is duplicated.
        inherited.unlock().unwrap();
        drop(inherited);
        let guard = ExclusiveLock::acquire(open(), "first").unwrap();
        let duplicate = guard.file.try_clone().unwrap();
        assert_eq!(
            ExclusiveLock::acquire(open(), "held").err().unwrap().kind,
            ErrorKind::Busy
        );
        drop(guard);
        let next = ExclusiveLock::acquire(open(), "released").unwrap();
        drop(next);
        drop(duplicate);
        std::fs::remove_file(path).unwrap();
    }
}
