use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActivityEntry {
    pub repository: RepositoryId,
    pub worktree: WorktreeId,
    pub commit: crate::activity::Commit,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActivityReport {
    pub items: Vec<ActivityEntry>,
    pub issues: Vec<Issue>,
    pub complete: bool,
    pub observed_at_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditPhase {
    pub phase: Phase,
    pub published: bool,
    pub new_oid: Option<String>,
    pub at_ms: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditEntry {
    pub outcome: Outcome,
    pub worktree: WorktreeId,
    pub kind: String,
    pub subject: String,
    pub created_at_ms: u64,
    pub events: Vec<AuditPhase>,
    pub events_complete: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditQuery {
    pub offset: usize,
    pub worktree: Option<String>,
    pub operation_kind: Option<String>,
    pub phase: Option<Phase>,
    pub since_ms: Option<u64>,
    pub until_ms: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditPage {
    pub items: Vec<AuditEntry>,
    pub next: Option<usize>,
}
impl Application {
    pub(super) fn activity(
        &self,
        workspace: &str,
        worktree: Option<&str>,
        since: u64,
        until: u64,
        reference: Option<String>,
        author: Option<String>,
    ) -> Result<ActivityReport> {
        let id = self.active(workspace)?;
        if until > i64::MAX as u64
            || since > until
            || until - since > 400 * 86400
            || author.as_ref().is_some_and(|a| a.len() > 512)
        {
            return Err(invalid("Invalid activity range or author filter"));
        }
        let entries = self.writes.workspace.inventory(&id)?;
        if worktree.is_some_and(|id| !entries.iter().any(|e| e.id.0 == id)) {
            return Err(invalid("Unknown activity worktree"));
        }
        let started = Instant::now();
        let mut seen = BTreeSet::new();
        let mut report = ActivityReport {
            items: Vec::new(),
            issues: Vec::new(),
            complete: true,
            observed_at_ms: now_ms(),
        };
        for entry in entries {
            if worktree.is_some_and(|id| entry.id.0 != id) || seen.contains(&entry.repository_id) {
                continue;
            }
            if self.read_options.cancellation.is_cancelled() {
                return Err(Error::with_kind(ErrorKind::Cancelled, "Activity cancelled"));
            }
            if report.items.len() == 5000 || started.elapsed() >= self.read_options.timeout {
                report.complete = false;
                break;
            }
            let result = (|| {
                let e = self.entry(workspace, &entry.id.0)?;
                let mut options = self.read_options.clone();
                options.timeout = options.timeout.saturating_sub(started.elapsed());
                let scope = reference
                    .clone()
                    .map(history::HistoryScope::Reference)
                    .unwrap_or(history::HistoryScope::LocalBranches);
                crate::activity::query(
                    &e.metadata.root.to_path()?,
                    scope,
                    since,
                    until,
                    5000 - report.items.len(),
                    &options,
                )
            })();
            match result {
                Ok(page) => {
                    seen.insert(entry.repository_id.clone());
                    report.complete &= page.complete;
                    report.items.extend(
                        page.commits
                            .into_iter()
                            .filter(|c| {
                                author.as_ref().is_none_or(|a| {
                                    String::from_utf8_lossy(&c.author)
                                        .to_lowercase()
                                        .contains(&a.to_lowercase())
                                })
                            })
                            .map(|commit| ActivityEntry {
                                repository: entry.repository_id.clone(),
                                worktree: entry.id.clone(),
                                commit,
                            }),
                    );
                }
                Err(error) if error.kind == ErrorKind::Cancelled => return Err(error),
                Err(error) => {
                    report.complete = false;
                    report.issues.push(error.into());
                }
            }
        }
        report.items.sort_by(|a, b| {
            b.commit
                .committer_time
                .cmp(&a.commit.committer_time)
                .then(a.repository.cmp(&b.repository))
                .then(a.commit.oid.hex.cmp(&b.commit.oid.hex))
        });
        if serde_json::to_vec(&report)?.len() > 8 * 1024 * 1024 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Activity response exceeds 8 MiB",
            ));
        }
        Ok(report)
    }
}
