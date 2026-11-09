//! Consistent SQLite copies between owner-provided connections, never live-file copying.
use crate::{inspection::Cancellation, Error, ErrorKind, Result};
use rusqlite::{
    backup::{Backup, StepResult},
    Connection,
};
use std::time::{Duration, Instant};
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::Storage, message)
}

/// The application owns the source and a fresh, private, empty staging connection.
/// Filesystem creation, durability and atomic publication remain the staging owner's
/// responsibility. This primitive never opens an arbitrary caller-supplied path.
pub fn database(
    source: &Connection,
    target: &mut Connection,
    cancel: &Cancellation,
) -> Result<u32> {
    if cancel.is_cancelled() {
        return Err(Error::with_kind(
            ErrorKind::Cancelled,
            "Backup cancelled before SQLite snapshot",
        ));
    }
    let existing: i64 =
        target.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))?;
    let version: u32 = target.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if existing != 0 || version != 0 {
        return Err(invalid(
            "SQLite snapshot requires a new empty staging database",
        ));
    }
    let page_size =
        u64::try_from(source.pragma_query_value(None, "page_size", |row| row.get::<_, i64>(0))?)
            .map_err(|_| invalid("Invalid SQLite page size"))?;
    let pages =
        u64::try_from(source.pragma_query_value(None, "page_count", |row| row.get::<_, i64>(0))?)
            .map_err(|_| invalid("Invalid SQLite page count"))?;
    if page_size == 0
        || pages
            .checked_mul(page_size)
            .is_none_or(|bytes| bytes > super::MAX_COMPONENT_BYTES)
    {
        return Err(invalid("SQLite snapshot exceeds backup component limit"));
    }
    {
        let copy = Backup::new(source, target)?;
        let start = Instant::now();
        loop {
            if cancel.is_cancelled() {
                return Err(Error::with_kind(
                    ErrorKind::Cancelled,
                    "SQLite snapshot cancelled",
                ));
            }
            if start.elapsed() > Duration::from_secs(30) {
                return Err(Error::with_kind(
                    ErrorKind::Timeout,
                    "SQLite snapshot exceeded its deadline",
                ));
            }
            if u64::try_from(copy.progress().pagecount)
                .ok()
                .and_then(|pages| pages.checked_mul(page_size))
                .is_none_or(|bytes| bytes > super::MAX_COMPONENT_BYTES)
            {
                return Err(invalid(
                    "SQLite snapshot grew beyond backup component limit",
                ));
            }
            match copy.step(64)? {
                StepResult::Done => {
                    if u64::try_from(copy.progress().pagecount)
                        .ok()
                        .and_then(|pages| pages.checked_mul(page_size))
                        .is_none_or(|bytes| bytes > super::MAX_COMPONENT_BYTES)
                    {
                        return Err(invalid("Completed SQLite snapshot exceeds component limit"));
                    }
                    break;
                }
                StepResult::More => {}
                StepResult::Busy | StepResult::Locked => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                _ => return Err(invalid("Unsupported SQLite backup state")),
            }
        }
    }
    let integrity: String = target.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(invalid("SQLite snapshot integrity check failed"));
    }
    target
        .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
        .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    #[test]
    fn wal_snapshot_is_consistent_and_keeps_original_database_bytes() {
        let root = std::env::temp_dir().join(format!(
            "gitfleet-snapshot-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let source = Connection::open(root.join("source.db")).unwrap();
        source.execute_batch("PRAGMA journal_mode=WAL; PRAGMA user_version=3; CREATE TABLE data(value TEXT); INSERT INTO data VALUES('committed WAL value');").unwrap();
        let original = fs::read(root.join("source.db")).unwrap();
        let wal = fs::read(root.join("source.db-wal")).unwrap();
        let mut target = Connection::open_in_memory().unwrap();
        assert_eq!(
            database(&source, &mut target, &Cancellation::default()).unwrap(),
            3
        );
        assert_eq!(
            target
                .query_row("SELECT value FROM data", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "committed WAL value"
        );
        assert_eq!(fs::read(root.join("source.db")).unwrap(), original);
        assert_eq!(fs::read(root.join("source.db-wal")).unwrap(), wal);
        drop(target);
        drop(source);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cancellation_and_nonempty_destination_preserve_previous_state() {
        let source = Connection::open_in_memory().unwrap();
        let mut target = Connection::open_in_memory().unwrap();
        target
            .execute_batch("CREATE TABLE keep(value TEXT); INSERT INTO keep VALUES('preserved');")
            .unwrap();
        assert!(database(&source, &mut target, &Cancellation::default()).is_err());
        assert_eq!(
            target
                .query_row("SELECT value FROM keep", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "preserved"
        );
        let mut empty = Connection::open_in_memory().unwrap();
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            database(&source, &mut empty, &cancel).unwrap_err().kind,
            ErrorKind::Cancelled
        );
        assert_eq!(
            empty
                .query_row("SELECT count(*) FROM sqlite_master", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
