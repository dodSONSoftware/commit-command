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
    pub(super) fn workspace_diff(
        &mut self,
        workspace: &str,
        worktree: &str,
        mode: diff::DiffMode,
        path: Option<&NativePath>,
    ) -> Result<diff::Diff> {
        let entry = self.entry(workspace, worktree)?;
        if !matches!(mode, diff::DiffMode::Unstaged) {
            return diff::inspect_with_options(
                &entry.metadata.root.to_path()?,
                mode,
                path,
                &self.read_options,
            );
        }
        let entry = self.writes.workspace.refresh_with_options(
            &WorkspaceId(workspace.into()),
            &entry.id,
            &self.read_options,
        )?;
        let root = entry.metadata.root.to_path()?;
        let mut result = diff::inspect_with_options(&root, mode, path, &self.read_options)?;
        let status = entry.status.as_ref().ok_or_else(stale)?;
        if result.generation != status.generation {
            return Err(stale());
        }
        let entries = self
            .writes
            .workspace
            .inventory(&WorkspaceId(workspace.into()))?;
        for link in status.index_gitlinks.as_deref().unwrap_or_default() {
            if path.is_some_and(|path| path != &link.path) {
                continue;
            }
            let child_root = root.join(link.path.relative()?);
            let Some(child) = entries.iter().find(|child| {
                child.metadata.root.to_path().ok().as_ref() == Some(&child_root)
                    && matches!(child.availability, Availability::Available)
                    && child.superseded_by.is_none()
            }) else {
                continue;
            };
            let Some(child_status) = child.status.as_ref() else {
                continue;
            };
            let Some(head) = child_status.head_oid.as_ref() else {
                continue;
            };
            if head == &link.oid || result.files.iter().any(|file| file.path == link.path) {
                continue;
            }
            // Compare independent observations; never ask parent Git to recurse into child configuration.
            self.entry(workspace, &child.id.0)?;
            let summary = format!("\nSubmodule: {}\nIndex commit: {}\nObserved child HEAD: {}\nComparison uses the child's last local inspection.\n", link.path.display(), link.oid.hex, head.hex);
            if result.patch.len().saturating_add(summary.len()) > self.read_options.max_output {
                return Err(Error::with_kind(
                    ErrorKind::OutputLimit,
                    "Diff exceeds output budget",
                ));
            }
            result.patch.extend_from_slice(summary.as_bytes());
            result.files.push(diff::FileChange {
                path: link.path.clone(),
                original_path: None,
                change: "M".into(),
                old_mode: "160000".into(),
                new_mode: "160000".into(),
                old_oid: Some(link.oid.clone()),
                new_oid: Some(head.clone()),
            });
            result.observed_at_ms = result.observed_at_ms.min(child_status.observed_at_ms);
        }
        Ok(result)
    }
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
