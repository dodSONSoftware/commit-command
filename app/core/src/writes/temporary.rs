//! Transient ownership receipts. A name alone is never deletion authority.
use super::*;
use std::os::unix::fs::MetadataExt;

#[derive(Serialize, Deserialize, PartialEq)]
struct Identity {
    dev: u64,
    ino: u64,
    created: Option<std::time::SystemTime>,
}
impl Identity {
    fn of(meta: &fs::Metadata) -> Self {
        Self {
            dev: meta.dev(),
            ino: meta.ino(),
            created: meta.created().ok(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Receipt {
    state: Identity,
    path: NativePath,
    parent: Identity,
    item: Identity,
    directory: bool,
}

fn key(id: &str) -> String {
    format!("write-temporary:{id}")
}

pub(super) fn remember(
    db: &rusqlite::Connection,
    operations: &Path,
    id: &str,
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| unsupported("Missing temporary parent"))?;
    let receipt = Receipt {
        state: Identity::of(&fs::metadata(operations)?),
        path: NativePath::from_path(path),
        parent: Identity::of(&fs::metadata(parent)?),
        item: Identity::of(metadata),
        directory: metadata.is_dir(),
    };
    db.execute(
        "INSERT INTO state(key,value) VALUES(?1,?2)",
        params![key(id), serde_json::to_string(&receipt)?],
    )?;
    Ok(())
}

pub(super) fn forget(db: &rusqlite::Connection, id: &str) -> Result<()> {
    db.execute("DELETE FROM state WHERE key=?1", [key(id)])?;
    Ok(())
}

pub(super) fn cleanup(db: &rusqlite::Connection, operations: &Path, id: &str) -> Result<()> {
    let raw: Option<String> = db
        .query_row("SELECT value FROM state WHERE key=?1", [key(id)], |r| {
            r.get(0)
        })
        .optional()?;
    let Some(raw) = raw else { return Ok(()) };
    Uuid::parse_str(id).map_err(|_| unsupported("Invalid temporary receipt ID"))?;
    let receipt: Receipt = serde_json::from_str(&raw)?;
    if receipt.state != Identity::of(&fs::metadata(operations)?) {
        // A copied database cannot authorize cleanup in the original workspace.
        return forget(db, id);
    }
    let path = receipt.path.to_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| unsupported("Invalid temporary receipt path"))?;
    if receipt.directory {
        if path != operations.join(id) {
            return Err(unsupported("Invalid scratch receipt path"));
        }
        if db.query_row(
            "SELECT EXISTS(SELECT 1 FROM write_operations WHERE id=?1)",
            [id],
            |r| r.get::<_, bool>(0),
        )? {
            return forget(db, id);
        }
    } else if path.file_name() != Some(std::ffi::OsStr::new(&format!(".gitfleet-object-{id}")))
        || parent.parent().and_then(Path::file_name) != Some(std::ffi::OsStr::new("objects"))
    {
        return Err(unsupported("Invalid object temporary receipt path"));
    }
    match fs::symlink_metadata(&path) {
        Ok(meta)
            if !meta.is_symlink()
                && receipt.directory == meta.is_dir()
                && Identity::of(&meta) == receipt.item
                && Identity::of(&fs::symlink_metadata(parent)?) == receipt.parent =>
        {
            if receipt.directory {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            store::sync_dir(parent)?;
        }
        Ok(_) => {} // A replacement belongs to someone else.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    forget(db, id)
}

pub(super) fn recover(db: &rusqlite::Connection, operations: &Path) -> Result<()> {
    let mut statement = db.prepare("SELECT key FROM state WHERE key GLOB 'write-temporary:*'")?;
    let keys = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    for k in keys {
        cleanup(db, operations, &k["write-temporary:".len()..])?;
    }
    Ok(())
}
