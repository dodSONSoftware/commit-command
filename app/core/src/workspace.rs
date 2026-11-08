//! TOML owns definitions; SQLite owns identity/inventory. No Git writes live here.
use crate::discovery::{self, DiscoveryOptions, Inventory};
use crate::domain::{digest, Issue, NativePath, RepositoryId, WorkspaceId, WorktreeId};
use crate::inspection::{self, Metadata, Snapshot};
use crate::{Error, ErrorKind, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub root: NativePath,
    pub display_root: NativePath,
    pub discovery: DiscoveryOptions,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
struct Config {
    version: u32,
    workspaces: Vec<Workspace>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            workspaces: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Missing,
    Unavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct StoredWorktree {
    pub id: WorktreeId,
    pub repository_id: RepositoryId,
    pub metadata: Metadata,
    pub display_path: NativePath,
    pub availability: Availability,
    pub status: Option<Snapshot>,
    pub issue: Option<Issue>,
    pub untrusted: bool,
    /// Historical identity replaced at the same path by a locally inspected identity.
    #[serde(default)]
    pub superseded_by: Option<WorktreeId>,
}
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct WorkspaceInventory {
    pub workspace_id: WorkspaceId,
    pub scan: Inventory,
    pub entries: Vec<StoredWorktree>,
}
pub struct WorkspaceService {
    directory: PathBuf,
    database: Connection,
    config: Config,
    _writer: crate::locking::ExclusiveLock,
}
fn storage(message: &str) -> Error {
    Error::with_kind(ErrorKind::Storage, message)
}
fn private_create(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}
fn reject_symlink(path: &Path) -> Result<()> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.is_symlink() || !meta.is_file() {
            return Err(storage(
                "Application state must be regular non-symlink files",
            ));
        }
    }
    Ok(())
}
impl WorkspaceService {
    pub fn open(directory: &Path) -> Result<Self> {
        // Refuse even creation of the state directory inside an existing checkout.
        for ancestor in directory.ancestors() {
            if ancestor.join(".git").exists() {
                return Err(Error::with_kind(
                    ErrorKind::InvalidPath,
                    "Application state must be outside managed repositories",
                ));
            }
        }
        fs::create_dir_all(directory)?;
        let directory = fs::canonicalize(directory)?;
        for ancestor in directory.ancestors() {
            if ancestor.join(".git").exists() {
                return Err(Error::with_kind(
                    ErrorKind::InvalidPath,
                    "Application state resolves inside a repository",
                ));
            }
        }
        for name in [
            "writer.lock",
            "config.toml",
            "state.db",
            "state.db-journal",
            "state.db-wal",
            "state.db-shm",
        ] {
            reject_symlink(&directory.join(name))?;
        }
        let mut open = OpenOptions::new();
        open.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            open.mode(0o600);
        }
        let writer = open.open(directory.join("writer.lock"))?;
        let writer = crate::locking::ExclusiveLock::acquire(
            writer,
            "GitFleet data directory already has a writer",
        )?;
        let config_path = directory.join("config.toml");
        let config = if config_path.exists() {
            let bytes = inspection::file_bytes(&config_path, 2 * 1024 * 1024)?;
            let text = std::str::from_utf8(&bytes).map_err(|_| storage("Invalid TOML encoding"))?;
            let value: toml::Value = toml::from_str(text)
                .map_err(|_| storage("Invalid workspace TOML; original preserved"))?;
            if value.get("version").and_then(toml::Value::as_integer) != Some(1) {
                return Err(Error::with_kind(
                    ErrorKind::NewerSchema,
                    "Unsupported configuration version",
                ));
            }
            toml::from_str::<Config>(text)
                .map_err(|_| storage("Invalid workspace configuration; original preserved"))?
        } else {
            Config::default()
        };
        validate_config(&config, &directory)?;
        let database_path = directory.join("state.db");
        let existed = database_path.exists();
        let mut database = Connection::open(&database_path)?;
        let version: u32 = database.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > 3 {
            return Err(Error::with_kind(
                ErrorKind::NewerSchema,
                "SQLite schema is newer than this Core",
            ));
        }
        database.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL; PRAGMA journal_mode=DELETE;",
        )?;
        if version == 0 {
            let tables:i64=database.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",[],|row|row.get(0))?;
            if tables != 0 {
                return Err(storage(
                    "Unversioned database has unknown tables; original preserved",
                ));
            }
            if existed {
                fs::copy(
                    &database_path,
                    directory.join(format!("state.pre-v1-{}.db", Uuid::new_v4())),
                )?;
            }
            let transaction = database.transaction()?;
            transaction.execute_batch("CREATE TABLE repositories(id TEXT PRIMARY KEY,binding TEXT NOT NULL UNIQUE);
                CREATE TABLE worktrees(id TEXT PRIMARY KEY,repository_id TEXT NOT NULL REFERENCES repositories(id),binding TEXT NOT NULL UNIQUE,metadata TEXT NOT NULL);
                CREATE TABLE membership(workspace_id TEXT NOT NULL,worktree_id TEXT NOT NULL REFERENCES worktrees(id),availability TEXT NOT NULL,status TEXT,issue TEXT,PRIMARY KEY(workspace_id,worktree_id));
                CREATE TABLE state(key TEXT PRIMARY KEY,value TEXT NOT NULL);
                PRAGMA user_version=1;")?;
            transaction.commit()?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&database_path, fs::Permissions::from_mode(0o600))?;
        }
        let mut service = Self {
            directory,
            _writer: writer,
            database,
            config,
        };
        service.reconcile()?;
        Ok(service)
    }
    #[cfg(feature = "controlled-writes")]
    pub(crate) fn connection(&mut self) -> &mut Connection {
        &mut self.database
    }
    #[cfg(feature = "controlled-writes")]
    pub(crate) fn data_directory(&self) -> &Path {
        &self.directory
    }
    fn reconcile(&mut self) -> Result<()> {
        let transaction = self.database.transaction()?;
        let ids = self
            .config
            .workspaces
            .iter()
            .map(|w| w.id.0.as_str())
            .collect::<Vec<_>>();
        let stale = {
            let mut statement =
                transaction.prepare("SELECT DISTINCT workspace_id FROM membership")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };
        for id in stale {
            if !ids.contains(&id.as_str()) {
                transaction.execute("DELETE FROM membership WHERE workspace_id=?1", [id])?;
            }
        }
        for workspace in &self.config.workspaces {
            let key = format!("definition:{}", workspace.id.0);
            let revision = digest(&serde_json::to_vec(&(
                &workspace.root,
                &workspace.discovery,
            ))?);
            let previous: Option<String> = transaction
                .query_row("SELECT value FROM state WHERE key=?1", [&key], |row| {
                    row.get(0)
                })
                .optional()?;
            if previous.as_ref() != Some(&revision) {
                transaction.execute(
                    "UPDATE membership SET availability='unavailable' WHERE workspace_id=?1",
                    [&workspace.id.0],
                )?;
                transaction.execute(
                    "DELETE FROM state WHERE key=?1",
                    [format!("scan:{}", workspace.id.0)],
                )?;
            }
            transaction.execute(
                "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
                params![key, revision],
            )?;
        }
        let active: Option<String> = transaction
            .query_row("SELECT value FROM state WHERE key='active'", [], |row| {
                row.get(0)
            })
            .optional()?;
        if active
            .as_ref()
            .is_some_and(|id| !ids.contains(&id.as_str()))
        {
            transaction.execute("DELETE FROM state WHERE key='active'", [])?;
        }
        transaction.execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES('config_revision',?1)",
            [digest(&serde_json::to_vec(&self.config)?)],
        )?;
        transaction.commit()?;
        Ok(())
    }
    fn save(&mut self, next: Config) -> Result<()> {
        validate_config(&next, &self.directory)?;
        let text = toml::to_string_pretty(&next)
            .map_err(|_| storage("Cannot serialize workspace configuration"))?;
        if text.len() > 2 * 1024 * 1024 {
            return Err(storage(
                "Workspace configuration exceeds its readable size limit",
            ));
        }
        let temporary = self
            .directory
            .join(format!("config-{}.tmp", Uuid::new_v4()));
        let result = (|| -> Result<()> {
            let mut file = private_create(&temporary)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, self.directory.join("config.toml"))?;
            File::open(&self.directory)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
        self.config = next;
        self.reconcile()
    }
    pub fn list(&self) -> &[Workspace] {
        &self.config.workspaces
    }
    pub fn create(&mut self, name: &str, root: &Path) -> Result<Workspace> {
        let display_root = NativePath::from_path(root);
        let root = fs::canonicalize(root)
            .map_err(|_| Error::with_kind(ErrorKind::Unavailable, "Workspace root unavailable"))?;
        if !root.is_dir() {
            return Err(Error::with_kind(
                ErrorKind::InvalidPath,
                "Workspace root must be a directory",
            ));
        }
        let workspace = Workspace {
            id: WorkspaceId(Uuid::new_v4().to_string()),
            name: name.to_owned(),
            root: NativePath::from_path(&root),
            display_root,
            discovery: DiscoveryOptions::default(),
        };
        let mut next = self.config.clone();
        next.workspaces.push(workspace.clone());
        self.save(next)?;
        Ok(workspace)
    }
    pub fn update(
        &mut self,
        id: &WorkspaceId,
        name: Option<&str>,
        root: Option<&Path>,
        options: Option<DiscoveryOptions>,
    ) -> Result<()> {
        let mut next = self.config.clone();
        let workspace = next
            .workspaces
            .iter_mut()
            .find(|w| &w.id == id)
            .ok_or_else(|| Error::with_kind(ErrorKind::InvalidRequest, "Unknown workspace"))?;
        if let Some(name) = name {
            workspace.name = name.to_owned();
        }
        if let Some(root) = root {
            workspace.display_root = NativePath::from_path(root);
            let root = fs::canonicalize(root)?;
            if !root.is_dir() {
                return Err(Error::with_kind(
                    ErrorKind::InvalidPath,
                    "Workspace root must be a directory",
                ));
            }
            workspace.root = NativePath::from_path(&root);
        }
        if let Some(options) = options {
            workspace.discovery = options;
        }
        self.save(next)
    }
    pub fn remove(&mut self, id: &WorkspaceId) -> Result<()> {
        let mut next = self.config.clone();
        let original = next.workspaces.len();
        next.workspaces.retain(|w| &w.id != id);
        if next.workspaces.len() == original {
            return Err(Error::with_kind(
                ErrorKind::InvalidRequest,
                "Unknown workspace",
            ));
        }
        self.save(next)
    }
    pub fn activate(&self, id: &WorkspaceId) -> Result<()> {
        self.definition(id)?;
        self.database.execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES('active',?1)",
            [&id.0],
        )?;
        Ok(())
    }
    pub fn active(&self) -> Result<Option<WorkspaceId>> {
        Ok(self
            .database
            .query_row("SELECT value FROM state WHERE key='active'", [], |row| {
                row.get(0)
            })
            .optional()?
            .map(WorkspaceId))
    }
    fn definition(&self, id: &WorkspaceId) -> Result<&Workspace> {
        self.config
            .workspaces
            .iter()
            .find(|w| &w.id == id)
            .ok_or_else(|| Error::with_kind(ErrorKind::InvalidRequest, "Unknown workspace"))
    }
    pub fn scan(&mut self, id: &WorkspaceId) -> Result<WorkspaceInventory> {
        self.scan_with_options(id, &inspection::ReadOptions::default())
    }
    pub fn scan_with_options(
        &mut self,
        id: &WorkspaceId,
        options: &inspection::ReadOptions,
    ) -> Result<WorkspaceInventory> {
        let workspace = self.definition(id)?.clone();
        let report = discovery::scan_with_options(
            &workspace.root.to_path()?,
            &workspace.discovery,
            options,
        )?;
        let previous = self.inventory(id)?;
        let transaction = self.database.transaction()?;
        transaction.execute(
            "UPDATE membership SET availability=?2 WHERE workspace_id=?1",
            params![
                id.0,
                if report.complete {
                    "missing"
                } else {
                    "unavailable"
                }
            ],
        )?;
        for entry in &report.repositories {
            let metadata = &entry.metadata;
            let repository_id: Option<String> = transaction
                .query_row(
                    "SELECT id FROM repositories WHERE binding=?1",
                    [&metadata.common_binding],
                    |row| row.get(0),
                )
                .optional()?;
            let repository_id = repository_id.unwrap_or_else(|| Uuid::new_v4().to_string());
            transaction.execute(
                "INSERT OR IGNORE INTO repositories(id,binding) VALUES(?1,?2)",
                params![repository_id, metadata.common_binding],
            )?;
            let binding = digest(&serde_json::to_vec(&(
                &metadata.root_binding,
                &metadata.git_binding,
                &metadata.common_binding,
            ))?);
            let worktree_id: Option<String> = transaction
                .query_row(
                    "SELECT id FROM worktrees WHERE binding=?1",
                    [&binding],
                    |row| row.get(0),
                )
                .optional()?;
            let worktree_id = worktree_id.unwrap_or_else(|| Uuid::new_v4().to_string());
            // A successful inspection establishes a current identity at this exact
            // path. Keep old IDs and their state; never transfer authority or drafts.
            if entry.status.is_some() {
                transaction.execute(
                    "DELETE FROM state WHERE key=?1",
                    [format!("superseded:{}:{}", id.0, worktree_id)],
                )?;
                for old in &previous {
                    if old.id.0 != worktree_id && old.metadata.root == metadata.root {
                        transaction.execute(
                            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
                            params![
                                format!("superseded:{}:{}", id.0, old.id.0),
                                serde_json::to_string(&WorktreeId(worktree_id.clone()))?
                            ],
                        )?;
                    }
                }
            }
            let serialized = serde_json::to_string(metadata)?;
            transaction.execute("INSERT INTO worktrees(id,repository_id,binding,metadata) VALUES(?1,?2,?3,?4) ON CONFLICT(binding) DO UPDATE SET metadata=excluded.metadata",params![worktree_id,repository_id,binding,serialized])?;
            transaction.execute("INSERT OR REPLACE INTO membership(workspace_id,worktree_id,availability,status,issue) VALUES(?1,?2,?3,?4,?5)",params![id.0,worktree_id,if entry.status.is_some(){"available"}else{"unavailable"},entry.status.as_ref().map(serde_json::to_string).transpose()?,entry.issue.as_ref().map(serde_json::to_string).transpose()?])?;
        }
        let serialized = serde_json::to_string(&report)?;
        if serialized.len() > 64 * 1024 * 1024 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Inventory cache exceeds budget",
            ));
        }
        transaction.execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![format!("scan:{}", id.0), serialized],
        )?;
        transaction.commit()?;
        Ok(WorkspaceInventory {
            workspace_id: id.clone(),
            scan: report,
            entries: self.inventory(id)?,
        })
    }
    pub fn last_scan(&self, id: &WorkspaceId) -> Result<Option<Inventory>> {
        self.definition(id)?;
        let value: Option<String> = self
            .database
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("scan:{}", id.0)],
                |row| row.get(0),
            )
            .optional()?;
        value
            .map(|value| serde_json::from_str(&value).map_err(Error::from))
            .transpose()
    }
    pub fn inventory(&self, id: &WorkspaceId) -> Result<Vec<StoredWorktree>> {
        let workspace_root = self.definition(id)?.root.to_path()?;
        let mut statement=self.database.prepare("SELECT w.id,w.repository_id,w.metadata,m.availability,m.status,m.issue FROM worktrees w JOIN membership m ON m.worktree_id=w.id WHERE m.workspace_id=?1 ORDER BY w.id")?;
        let rows = statement.query_map([&id.0], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;
        let workspace_id = &id.0;
        rows.map(|row| {
            let (id, repository_id, metadata, availability, status, issue) = row?;
            let metadata: Metadata = serde_json::from_str(&metadata)?;
            let root = metadata.root.to_path()?;
            let display_path =
                NativePath::from_path(root.strip_prefix(&workspace_root).unwrap_or(&root));
            Ok(StoredWorktree {
                display_path,
                id: WorktreeId(id.clone()),
                repository_id: RepositoryId(repository_id),
                metadata,
                availability: match availability.as_str() {
                    "available" => Availability::Available,
                    "missing" => Availability::Missing,
                    _ => Availability::Unavailable,
                },
                status: status.map(|s| serde_json::from_str(&s)).transpose()?,
                issue: issue.map(|s| serde_json::from_str(&s)).transpose()?,
                untrusted: true,
                superseded_by: self
                    .database
                    .query_row(
                        "SELECT value FROM state WHERE key=?1",
                        [format!("superseded:{}:{}", workspace_id, id)],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?
                    .map(|value| serde_json::from_str(&value))
                    .transpose()?,
            })
        })
        .collect()
    }
    /// A targeted status refresh does not recursively rediscover the workspace.
    pub fn refresh(&mut self, workspace: &WorkspaceId, id: &WorktreeId) -> Result<StoredWorktree> {
        self.refresh_with_options(workspace, id, &inspection::ReadOptions::default())
    }
    pub fn refresh_with_options(
        &mut self,
        workspace: &WorkspaceId,
        id: &WorktreeId,
        options: &inspection::ReadOptions,
    ) -> Result<StoredWorktree> {
        let old = self
            .inventory(workspace)?
            .into_iter()
            .find(|entry| &entry.id == id)
            .ok_or_else(|| Error::with_kind(ErrorKind::InvalidRequest, "Unknown worktree"))?;
        if old.superseded_by.is_some() {
            return Err(Error::with_kind(
                ErrorKind::Unavailable,
                "Historical repository identity; rescan to inspect the current repository",
            ));
        }
        let root = old.metadata.root.to_path()?;
        let workspace_root = self.definition(workspace)?.root.to_path()?;
        let refreshed = (|| -> Result<Snapshot> {
            if !root.starts_with(&workspace_root) {
                return Err(Error::with_kind(
                    ErrorKind::InvalidPath,
                    "Worktree is outside the updated workspace root; rescan required",
                ));
            }
            let actual = inspection::repository_metadata_with_options(&root, options)?;
            if actual.root_binding != old.metadata.root_binding
                || actual.git_binding != old.metadata.git_binding
                || actual.common_binding != old.metadata.common_binding
            {
                return Err(Error::with_kind(
                    ErrorKind::ConcurrentChange,
                    "Worktree identity was replaced; rescan required",
                ));
            }
            inspection::snapshot_with_options(&root, options)
        })();
        let status = match refreshed {
            Ok(status) => status,
            Err(error) => {
                let issue = Issue {
                    kind: error.kind,
                    message: error.to_string(),
                };
                self.database.execute("UPDATE membership SET availability='unavailable',issue=?3 WHERE workspace_id=?1 AND worktree_id=?2",params![workspace.0,id.0,serde_json::to_string(&issue)?])?;
                return Err(error);
            }
        };
        self.database.execute("UPDATE membership SET status=?3,issue=NULL,availability='available' WHERE workspace_id=?1 AND worktree_id=?2",params![workspace.0,id.0,serde_json::to_string(&status)?])?;
        Ok(StoredWorktree {
            status: Some(status),
            availability: Availability::Available,
            issue: None,
            ..old
        })
    }
}
fn validate_config(config: &Config, directory: &Path) -> Result<()> {
    let mut ids = std::collections::BTreeSet::new();
    for workspace in &config.workspaces {
        if Uuid::parse_str(&workspace.id.0).is_err()
            || !ids.insert(&workspace.id.0)
            || workspace.name.trim().is_empty()
        {
            return Err(storage("Invalid/duplicate workspace ID or empty name"));
        }
        let root = workspace.root.to_path()?;
        if !root.is_absolute() || directory.starts_with(&root) {
            return Err(Error::with_kind(
                ErrorKind::InvalidPath,
                "Workspace root must be absolute and outside application state",
            ));
        }
        workspace.discovery.validate()?;
    }
    Ok(())
}
