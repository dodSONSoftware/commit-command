use super::*;
use rusqlite::{params, OptionalExtension};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RepositoryTab {
    pub worktree: String,
    pub view: String,
    pub path: Option<NativePath>,
    pub commit: Option<String>,
    pub parent: Option<String>,
    pub source: ContentSource,
    pub scroll: f64,
    pub filter: String,
}
impl Default for RepositoryTab {
    fn default() -> Self {
        Self {
            worktree: String::new(),
            view: "changes".into(),
            path: None,
            commit: None,
            parent: None,
            source: ContentSource::WorkingTree,
            scroll: 0.,
            filter: String::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesktopState {
    pub version: u32,
    pub tabs: Vec<RepositoryTab>,
    pub active_tab: Option<String>,
    pub selected: Vec<String>,
    pub drafts: BTreeMap<String, String>,
    pub filter: String,
    pub layout: String,
    pub sort: String,
    pub theme: String,
    pub sidebar_width: u32,
    pub inspector_width: u32,
}
impl Default for DesktopState {
    fn default() -> Self {
        Self {
            version: 1,
            tabs: Vec::new(),
            active_tab: None,
            selected: Vec::new(),
            drafts: BTreeMap::new(),
            filter: String::new(),
            layout: "table".into(),
            sort: "path".into(),
            theme: "system".into(),
            sidebar_width: 240,
            inspector_width: 400,
        }
    }
}
pub(super) fn migrate(writes: &mut ControlledWrites) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let dir = writes.workspace.data_directory().to_path_buf();
    let db = writes.workspace.connection();
    let version: u32 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version == 2 {
        let backup = dir.join(format!("state.pre-m3-{}.db", Uuid::new_v4()));
        db.backup(
            rusqlite::MAIN_DB,
            &backup,
            None::<fn(rusqlite::backup::Progress)>,
        )?;
        std::fs::set_permissions(&backup, std::fs::Permissions::from_mode(0o600))?;
        std::fs::File::open(&backup)?.sync_all()?;
        std::fs::File::open(&dir)?.sync_all()?;
        let tx = db.transaction()?;
        tx.execute_batch("CREATE TABLE desktop_state(workspace_id TEXT PRIMARY KEY,payload TEXT NOT NULL); CREATE INDEX write_operations_workspace ON write_operations(json_extract(payload,'$.proposal.workspace_id')); PRAGMA user_version=3;")?;
        tx.commit()?;
    } else if version != 3 {
        return Err(Error::with_kind(
            ErrorKind::NewerSchema,
            "Unsupported desktop schema",
        ));
    }
    writes.workspace.connection().execute_batch("CREATE INDEX IF NOT EXISTS write_operations_workspace ON write_operations(json_extract(payload,'$.proposal.workspace_id'));")?;
    Ok(())
}
pub(super) fn load(writes: &mut ControlledWrites, id: &WorkspaceId) -> Result<DesktopState> {
    let raw: Option<String> = writes
        .workspace
        .connection()
        .query_row(
            "SELECT payload FROM desktop_state WHERE workspace_id=?1",
            [&id.0],
            |r| r.get(0),
        )
        .optional()?;
    let state = raw
        .map(|s| serde_json::from_str::<DesktopState>(&s))
        .transpose()?
        .unwrap_or_default();
    validate(&state)?;
    Ok(state)
}
pub(super) fn validate(s: &DesktopState) -> Result<()> {
    if s.version != 1 {
        return Err(Error::with_kind(
            ErrorKind::NewerSchema,
            "Unsupported navigation schema",
        ));
    }
    if !(180..=420).contains(&s.sidebar_width)
        || !(240..=800).contains(&s.inspector_width)
        || !["path", "changes"].contains(&s.sort.as_str())
        || s.tabs.len() > 100
        || s.selected.len() > 10000
        || s.drafts.len() > 100
        || !["system", "light", "dark"].contains(&s.theme.as_str())
        || !["table", "tree"].contains(&s.layout.as_str())
        || s.tabs.iter().any(|t| {
            !["changes", "history"].contains(&t.view.as_str())
                || !t.scroll.is_finite()
                || t.scroll < 0.
        })
        || s.drafts.values().any(|s| s.len() > 512 * 1024)
        || serde_json::to_vec(s)?.len() > 2 * 1024 * 1024
    {
        return Err(invalid("Invalid or oversized navigation state"));
    }
    let unique = s
        .tabs
        .iter()
        .map(|t| &t.worktree)
        .collect::<std::collections::BTreeSet<_>>();
    if unique.len() != s.tabs.len()
        || s.active_tab.as_ref().is_some_and(|id| !unique.contains(id))
        || s.selected
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != s.selected.len()
    {
        return Err(invalid("Duplicate or invalid tab/selection state"));
    }
    for tab in &s.tabs {
        if let Some(path) = &tab.path {
            path.relative()?;
        }
    }
    Ok(())
}
pub(super) fn save(
    writes: &mut ControlledWrites,
    id: &WorkspaceId,
    s: &DesktopState,
) -> Result<()> {
    validate(s)?;
    writes.workspace.connection().execute("INSERT INTO desktop_state(workspace_id,payload) VALUES(?1,?2) ON CONFLICT(workspace_id) DO UPDATE SET payload=excluded.payload",params![id.0,serde_json::to_string(s)?])?;
    Ok(())
}

/// Native presentation geometry in physical desktop coordinates. Never an IPC request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowPlacement {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
impl WindowPlacement {
    fn validate(&self) -> Result<()> {
        if self.x.unsigned_abs() > 100_000
            || self.y.unsigned_abs() > 100_000
            || !(200..=32_768).contains(&self.width)
            || !(200..=32_768).contains(&self.height)
        {
            return Err(invalid("Invalid native window placement"));
        }
        Ok(())
    }
}
impl Application {
    pub fn window_placement(&mut self) -> Result<Option<WindowPlacement>> {
        let raw: Option<String> = self
            .writes
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key='native_window_placement'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let placement = raw
            .map(|s| serde_json::from_str::<WindowPlacement>(&s))
            .transpose()?;
        if let Some(p) = &placement {
            p.validate()?;
        }
        Ok(placement)
    }
    pub fn save_window_placement(&mut self, placement: WindowPlacement) -> Result<()> {
        placement.validate()?;
        self.writes.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES('native_window_placement',?1)",
            [serde_json::to_string(&placement)?],
        )?;
        Ok(())
    }
}
