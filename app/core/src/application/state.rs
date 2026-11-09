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
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverviewFilter {
    #[default]
    All,
    Modified,
    Staged,
    Unstaged,
    Untracked,
    Conflicted,
    Ahead,
    Behind,
    Issues,
}
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverviewDensity {
    #[default]
    Comfortable,
    Compact,
}
/// Additive string variants preserve version-1 path/changes navigation records.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverviewSort {
    Path,
    PathDesc,
    Name,
    NameDesc,
    Branch,
    BranchDesc,
    Staged,
    StagedDesc,
    Unstaged,
    UnstagedDesc,
    Untracked,
    UntrackedDesc,
    Conflicts,
    ConflictsDesc,
    Ahead,
    AheadDesc,
    Behind,
    BehindDesc,
    LastCommit,
    LastCommitDesc,
    Trust,
    TrustDesc,
    Status,
    StatusDesc,
    Changes,
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
    pub sort: OverviewSort,
    #[serde(default)]
    pub attention: OverviewFilter,
    #[serde(default)]
    pub collapsed: Vec<String>,
    #[serde(default)]
    pub density: OverviewDensity,
    #[serde(default)]
    pub sidebar_collapsed: bool,
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
            sort: OverviewSort::Path,
            attention: OverviewFilter::All,
            collapsed: Vec::new(),
            density: OverviewDensity::Comfortable,
            sidebar_collapsed: false,
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
        || s.collapsed.len() > 10_000
        || s.collapsed
            .iter()
            .any(|key| key.is_empty() || key.len() > 4096 || key.contains('\0'))
        || s.collapsed
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != s.collapsed.len()
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

#[cfg(test)]
mod sort_tests {
    use super::*;
    #[test]
    fn legacy_sort_values_remain_compatible_and_unknown_values_fail_closed() {
        let mut value = serde_json::to_value(DesktopState::default()).unwrap();
        value.as_object_mut().unwrap().remove("attention");
        value.as_object_mut().unwrap().remove("collapsed");
        value.as_object_mut().unwrap().remove("density");
        value.as_object_mut().unwrap().remove("sidebar_collapsed");
        for (raw, expected) in [
            ("path", OverviewSort::Path),
            ("changes", OverviewSort::Changes),
            ("last_commit_desc", OverviewSort::LastCommitDesc),
        ] {
            value["sort"] = serde_json::json!(raw);
            let restored: DesktopState = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(restored.sort, expected);
            assert_eq!(restored.attention, OverviewFilter::All);
            assert!(restored.collapsed.is_empty());
            assert_eq!(restored.density, OverviewDensity::Comfortable);
            assert!(!restored.sidebar_collapsed);
            validate(&restored).unwrap();
            assert_eq!(serde_json::to_value(restored).unwrap()["sort"], raw);
        }
        value["sort"] = serde_json::json!("unsupported");
        assert!(serde_json::from_value::<DesktopState>(value).is_err());
    }
    #[test]
    fn collapse_state_is_bounded_and_cannot_duplicate_keys() {
        let mut state = DesktopState::default();
        for keys in [
            vec![String::new()],
            vec!["nul\0key".into()],
            vec!["x".repeat(4097)],
            vec!["/same".into(), "/same".into()],
        ] {
            state.collapsed = keys;
            assert!(validate(&state).is_err());
        }
        state.collapsed = (0..10_000).map(|n| format!("/group-{n}")).collect();
        validate(&state).unwrap();
        state.collapsed.push("/overflow".into());
        assert!(validate(&state).is_err());
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
