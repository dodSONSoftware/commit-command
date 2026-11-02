use super::*;
use crate::application::{OperationPage, TrustState};
impl ControlledWrites {
    pub(crate) fn application_remotes(
        &self,
        entry: &crate::workspace::StoredWorktree,
        options: &ReadOptions,
    ) -> Result<crate::remote::Inspection> {
        crate::remote::inspect(
            &entry.metadata.root.to_path()?,
            self.options.global_config.as_deref(),
            options,
        )
    }
    pub(crate) fn cancel_application_reviews(&mut self) {
        self.trust_reviews.clear();
        self.recovery_reviews.clear();
    }

    pub(crate) fn application_trust_state(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
    ) -> Result<TrustState> {
        let row: Option<(String, bool)> = self
            .workspace
            .connection()
            .query_row(
                "SELECT review,revoked FROM write_trust WHERE worktree_id=?1",
                [&entry.id.0],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((raw, revoked)) = row else {
            return Ok(TrustState {
                worktree: entry.id.0.clone(),
                grant: "untrusted".into(),
                validity: "none".into(),
                eligible: false,
                reason: Some("Explicit worktree trust required".into()),
                policy: None,
            });
        };
        let previous: TrustReview = serde_json::from_str(&raw)?;
        let mut result = TrustState {
            worktree: entry.id.0.clone(),
            grant: if revoked { "revoked" } else { "granted" }.into(),
            validity: "unverifiable".into(),
            eligible: false,
            reason: None,
            policy: self.hook_policy(&entry.id).ok(),
        };
        if revoked {
            result.validity = "revoked".into();
            return Ok(result);
        }
        match self.inspect_trust(workspace, &entry.id) {
            Ok(current) => {
                self.trust_reviews.remove(&current.review_id);
                let same = previous.binding == current.binding
                    && previous.config_hash == current.config_hash
                    && previous.hooks == current.hooks
                    && previous.signing == current.signing
                    && previous.signer_home == current.signer_home;
                result.validity = if same { "current" } else { "stale" }.into();
                result.eligible = same;
                if same {
                    if let Err(error) = policy::validate_hook_manifest(
                        &current.hooks,
                        result.policy.unwrap_or(HookPolicy::Execute),
                    ) {
                        result.eligible = false;
                        result.reason = Some(error.to_string());
                    }
                }
                if !same {
                    self.revoke_trust(&entry.id)?;
                    result.grant = "revoked".into();
                    result.reason = Some("Trust binding changed; review again".into());
                }
            }
            Err(e) => result.reason = Some(format!("{:?}: {}", e.kind, e)),
        };
        Ok(result)
    }
    pub(crate) fn application_operation_scope(
        &mut self,
        workspace: &WorkspaceId,
        id: &str,
    ) -> Result<()> {
        if self.load(id)?.proposal.workspace_id != *workspace {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Operation belongs to another workspace",
            ));
        }
        Ok(())
    }
    pub(crate) fn application_operations(
        &mut self,
        workspace: &WorkspaceId,
        offset: usize,
    ) -> Result<OperationPage> {
        if offset > 1_000_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Operation offset limit exceeded",
            ));
        }
        // Bound records read from SQLite rather than loading the entire history.
        let mut statement=self.workspace.connection().prepare("SELECT payload FROM write_operations WHERE json_extract(payload,'$.proposal.workspace_id')=?1 ORDER BY rowid DESC LIMIT 51 OFFSET ?2")?;
        let rows = statement.query_map(params![workspace.0, offset as i64], |r| {
            r.get::<_, String>(0)
        })?;
        let mut items = rows
            .map(|r| Ok(serde_json::from_str::<Operation>(&r?)?.outcome()))
            .collect::<Result<Vec<_>>>()?;
        let next = if items.len() > 50 {
            items.truncate(50);
            Some(offset + 50)
        } else {
            None
        };
        Ok(OperationPage { items, next })
    }
    pub fn execute_with_cancellation(
        &mut self,
        approval: &Approval,
        cancellation: inspection::Cancellation,
    ) -> Result<Outcome> {
        let previous = std::mem::replace(&mut self.options.process.cancellation, cancellation);
        let result = self.execute(approval);
        self.options.process.cancellation = previous;
        result
    }
}

impl ControlledWrites {
    pub(crate) fn application_cancel_proposal(&mut self, id: &str) -> Result<()> {
        let mut op = self.load(id)?;
        if matches!(op.phase, Phase::Proposed | Phase::Approved) {
            op.phase = Phase::CancelledBeforePublication;
            op.reason = Some(
                "Application review was cancelled or replaced; no execution was started".into(),
            );
            self.persist(&op)?;
        }
        Ok(())
    }
    pub(crate) fn application_pending(
        &mut self,
    ) -> Result<Vec<crate::application::RecoveryNotice>> {
        let mut statement=self.workspace.connection().prepare("SELECT payload FROM write_operations WHERE json_extract(payload,'$.phase') NOT IN ('complete','published_with_post_processing_failure','cancelled_before_publication','proposed','approved','needs_reauthorization') ORDER BY rowid DESC LIMIT 100")?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|r| {
            let op: Operation = serde_json::from_str(&r?)?;
            Ok(crate::application::RecoveryNotice {
                operation_id: op.proposal.operation_id,
                workspace_id: op.proposal.workspace_id.0,
                worktree_id: op.proposal.worktree_id.0,
                phase: op.phase,
                published: op.published,
                reason: op.reason,
            })
        })
        .collect()
    }
}
impl ControlledWrites {
    pub(crate) fn application_has_pending(&mut self, workspace: &str) -> Result<bool> {
        self.workspace.connection().query_row("SELECT EXISTS(SELECT 1 FROM write_operations WHERE json_extract(payload,'$.proposal.workspace_id')=?1 AND json_extract(payload,'$.phase') NOT IN ('complete','published_with_post_processing_failure','cancelled_before_publication','proposed','approved','needs_reauthorization'))",[workspace],|r|r.get(0)).map_err(Error::from)
    }
}

impl ControlledWrites {
    pub(crate) fn application_audit(
        &mut self,
        workspace: &WorkspaceId,
        query: crate::application::AuditQuery,
    ) -> Result<crate::application::AuditPage> {
        let crate::application::AuditQuery {
            offset,
            worktree,
            operation_kind: kind,
            phase,
            since_ms,
            until_ms,
        } = query;
        if since_ms.is_some_and(|v| v > i64::MAX as u64)
            || until_ms.is_some_and(|v| v > i64::MAX as u64)
            || since_ms.zip(until_ms).is_some_and(|(a, b)| a > b)
        {
            return Err(fail(ErrorKind::InvalidRequest, "Invalid audit time range"));
        }
        use crate::application::{AuditEntry, AuditPage, AuditPhase};
        if offset > 1_000_000
            || kind
                .as_ref()
                .is_some_and(|s| !["stage", "unstage", "commit"].contains(&s.as_str()))
        {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Invalid audit offset or operation kind",
            ));
        }
        let phase = phase
            .map(serde_json::to_value)
            .transpose()?
            .and_then(|v| v.as_str().map(str::to_owned));
        let payloads = {
            let mut statement = self.workspace.connection().prepare("SELECT payload FROM write_operations WHERE json_extract(payload,'$.proposal.workspace_id')=?1 AND (?2 IS NULL OR json_extract(payload,'$.proposal.worktree_id')=?2) AND (?3 IS NULL OR json_extract(payload,'$.proposal.request.kind')=?3) AND (?4 IS NULL OR json_extract(payload,'$.phase')=?4) AND (?6 IS NULL OR json_extract(payload,'$.proposal.created_at_ms')>=?6) AND (?7 IS NULL OR json_extract(payload,'$.proposal.created_at_ms')<=?7) ORDER BY rowid DESC LIMIT 51 OFFSET ?5")?;
            let rows = statement.query_map(
                params![
                    workspace.0,
                    worktree,
                    kind,
                    phase,
                    offset as i64,
                    since_ms.map(|v| v as i64),
                    until_ms.map(|v| v as i64)
                ],
                |r| r.get::<_, String>(0),
            )?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };
        let next = (payloads.len() > 50).then_some(offset + 50);
        let mut items = Vec::new();
        for payload in payloads.into_iter().take(50) {
            let op: Operation = serde_json::from_str(&payload)?;
            let mut events = {
                let mut statement = self.workspace.connection().prepare("SELECT phase,published,new_oid,at_ms FROM write_events WHERE operation_id=?1 ORDER BY sequence LIMIT 1001")?;
                let rows = statement.query_map([&op.proposal.operation_id], |r| {
                    Ok(AuditPhase {
                        phase: {
                            let text: String = r.get(0)?;
                            serde_json::from_str(&text).map_err(|e| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    0,
                                    rusqlite::types::Type::Text,
                                    Box::new(e),
                                )
                            })?
                        },
                        published: r.get(1)?,
                        new_oid: r.get(2)?,
                        at_ms: r.get(3)?,
                    })
                })?;
                rows.collect::<std::result::Result<Vec<_>, _>>()?
            };
            let events_complete = events.len() <= 1000;
            events.truncate(1000);
            let kind = match op.proposal.request {
                WriteRequest::Commit { .. } => "commit",
                WriteRequest::Stage { .. } => "stage",
                WriteRequest::Unstage { .. } => "unstage",
            };
            items.push(AuditEntry {
                outcome: op.outcome(),
                worktree: op.proposal.worktree_id,
                kind: kind.into(),
                subject: String::from_utf8_lossy(&op.proposal.message)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(512)
                    .collect(),
                created_at_ms: op.proposal.created_at_ms,
                events,
                events_complete,
            });
        }
        Ok(AuditPage { items, next })
    }
}

impl ControlledWrites {
    pub(crate) fn application_proposal(&mut self, id: &str) -> Result<Proposal> {
        Ok(self.load(id)?.proposal)
    }
    pub(crate) fn application_propose(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        request: WriteRequest,
        cancellation: inspection::Cancellation,
    ) -> Result<Proposal> {
        let previous = std::mem::replace(&mut self.options.process.cancellation, cancellation);
        let result = self.propose(workspace, worktree, request);
        self.options.process.cancellation = previous;
        result
    }
}
