//! Workspace bulk-composer preferences are state, never write authority.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchConfiguration {
    pub mode: BatchMode,
    pub excluded: Vec<NativePath>,
    pub hooks: HookPolicy,
    pub signing: SigningPreference,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BulkComposerState {
    pub version: u32,
    pub message: String,
    pub configurations: BTreeMap<String, BatchConfiguration>,
}
impl Default for BulkComposerState {
    fn default() -> Self {
        Self {
            version: 1,
            message: String::new(),
            configurations: BTreeMap::new(),
        }
    }
}
impl BulkComposerState {
    fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(Error::with_kind(
                ErrorKind::NewerSchema,
                "Unsupported bulk-composer state version",
            ));
        }
        if self.message.len() > 512 * 1024
            || self.configurations.len() > 100
            || serde_json::to_vec(self)?.len() > 2 * 1024 * 1024
        {
            return Err(invalid("Bulk-composer state exceeds size limits"));
        }
        for (id, configuration) in &self.configurations {
            if id.is_empty() || id.len() > 128 || configuration.excluded.len() > 10_000 {
                return Err(invalid("Invalid bulk-composer configuration"));
            }
            let mut seen = std::collections::BTreeSet::new();
            for path in &configuration.excluded {
                let relative = path.relative()?;
                if !seen.insert(relative.as_os_str().as_encoded_bytes().to_vec()) {
                    return Err(invalid("Duplicate excluded path"));
                }
            }
        }
        Ok(())
    }
}
impl Application {
    /// Save the final renderer snapshot through the existing exclusive storage owner.
    /// This carries presentation preferences only and never grants Git authority.
    pub fn save_presentation(
        &mut self,
        epoch: u64,
        workspace: &str,
        navigation: &DesktopState,
        bulk: &BulkComposerState,
    ) -> Result<()> {
        if epoch != self.epoch {
            return Err(stale());
        }
        let id = self.active(workspace)?;
        state::validate(navigation)?;
        bulk.validate()?;
        let navigation = serde_json::to_string(navigation)?;
        let bulk = serde_json::to_string(bulk)?;
        let tx = self.writes.workspace.connection().transaction()?;
        tx.execute(
            "INSERT INTO desktop_state(workspace_id,payload) VALUES(?1,?2) ON CONFLICT(workspace_id) DO UPDATE SET payload=excluded.payload",
            rusqlite::params![id.0, navigation],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            rusqlite::params![format!("bulk-composer:{}", id.0), bulk],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(super) fn load_bulk_composer(&mut self, workspace: &str) -> Result<BulkComposerState> {
        let id = self.active(workspace)?;
        use rusqlite::OptionalExtension;
        let raw: Option<String> = self
            .writes
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("bulk-composer:{}", id.0)],
                |r| r.get(0),
            )
            .optional()?;
        let state = match raw {
            Some(raw) if raw.len() <= 2 * 1024 * 1024 => serde_json::from_str(&raw)?,
            Some(_) => return Err(invalid("Stored bulk-composer state exceeds size limits")),
            None => BulkComposerState::default(),
        };
        state.validate()?;
        Ok(state)
    }
    pub(super) fn save_bulk_composer(
        &mut self,
        workspace: &str,
        state: &BulkComposerState,
    ) -> Result<()> {
        let id = self.active(workspace)?;
        state.validate()?;
        self.writes.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            rusqlite::params![
                format!("bulk-composer:{}", id.0),
                serde_json::to_string(state)?
            ],
        )?;
        Ok(())
    }
}
