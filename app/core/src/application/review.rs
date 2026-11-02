use super::*;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReviewDisplay {
    pub id: String,
    pub epoch: u64,
    pub expires_at_ms: u64,
    pub title: String,
    pub summary: String,
    pub content: String,
}
pub(super) enum Action {
    Trust {
        workspace: WorkspaceId,
        reviews: Vec<TrustReview>,
    },
    Write(Box<Proposal>),
    Batch(BatchRecord),
    Recovery {
        workspace: WorkspaceId,
        review: RecoveryReview,
    },
}
pub(super) struct Pending {
    pub id: String,
    pub epoch: u64,
    pub created: Instant,
    pub action: Action,
    pub display: ReviewDisplay,
    pub nonce: Option<String>,
}
/// Native-host capability. Never serialized or returned to the renderer.
pub struct Confirmation {
    id: String,
    epoch: u64,
    nonce: String,
}
impl Application {
    pub(super) fn describe_write(&self, p: &Proposal) -> Result<(String, String, String)> {
        let entry = self.entry(&p.workspace_id.0, &p.worktree_id.0)?;
        let (kind, paths) = match &p.request {
            WriteRequest::Commit { selected, .. } => (
                "COMMIT",
                selected
                    .iter()
                    .map(|s| format!("{} — {:?}", s.path.display(), s.source))
                    .collect::<Vec<_>>(),
            ),
            WriteRequest::Stage { paths, .. } => (
                "STAGE",
                paths
                    .iter()
                    .map(|p| format!("{} — working tree → index", p.display()))
                    .collect(),
            ),
            WriteRequest::Unstage { paths, .. } => (
                "UNSTAGE",
                paths
                    .iter()
                    .map(|p| format!("{} — HEAD → index", p.display()))
                    .collect(),
            ),
        };
        let mut summary = format!(
                    "{kind}\nWorktree: {}\nRepository: {}\nBranch: {}\nSelected paths: {}\nTree: {}\nHooks: {:?}",
                    entry.metadata.root.display(), p.repository_id.0, p.branch, paths.len(), p.approved_tree, p.hooks
                );
        let mut detail = format!(
                    "{summary}\n\nExpected HEAD: {}\nOperation: {}\nReview digest: {}\n\nSelected content\n{}\n",
                    p.expected_head.as_deref().unwrap_or("Unborn branch"), p.operation_id, p.review_digest, paths.join("\n")
                );
        if let WriteRequest::Commit { .. } = &p.request {
            let message = String::from_utf8_lossy(&p.message);
            let subject = message
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(160)
                .collect::<String>();
            let signer = p
                .signing
                .as_ref()
                .map(|s| s.fingerprint.as_str())
                .unwrap_or("UNSIGNED");
            summary.push_str(&format!(
                "\nMessage subject: {}\nSigner: {}",
                serde_json::to_string(&subject)?,
                signer
            ));
            detail.push_str(&format!(
                        "\nCommit identity\nAuthor: {} <{}>\nDate: {}\nSigner: {}\n\nExact message (JSON escaped)\n{}\n",
                        p.author_name, p.author_email, p.author_date, signer, serde_json::to_string(&message)?
                    ));
            if let Some(signing) = &p.signing {
                detail.push_str(&format!(
                    "\nSigning configuration\n{}\n",
                    serde_json::to_string_pretty(signing)?
                ));
            }
        } else {
            summary.push_str("\nHEAD and working files are preserved.");
        }
        detail.push_str("\nReviewed hook manifest\n");
        for hook in &p.hook_manifest {
            detail.push_str(&format!("{}\n", serde_json::to_string_pretty(hook)?));
        }
        detail.push_str("\nProposed patch\n");
        match std::str::from_utf8(&p.patch) {
            Ok(patch) => detail.push_str(patch),
            Err(_) => {
                detail.push_str("Non-UTF-8 patch; exact bytes (JSON)\n");
                detail.push_str(&serde_json::to_string(&p.patch)?);
            }
        }
        summary.push_str(&format!(
            "\nReview digest: {}\nFull content is in the authoritative review window.",
            p.review_digest
        ));
        Ok((format!("Review {}", kind.to_lowercase()), summary, detail))
    }
    pub(super) fn set_review(&mut self, action: Action) -> Result<ReviewDisplay> {
        let id = Uuid::new_v4().to_string();
        let (title, summary, content) = match &action {
            Action::Write(p) => self.describe_write(p)?,
            Action::Batch(record) => self.describe_batch(record)?,
            Action::Trust { reviews, .. } => (
                "Review explicit trust grants".into(),
                format!("Trust {} explicit worktree(s). Reviewed hooks/signers run unsandboxed as your user.\n{}", reviews.len(), reviews.iter().map(|r| format!("{}: {:?}", r.worktree_id.0, r.metadata.root)).collect::<Vec<_>>().join("\n")),
                serde_json::to_string_pretty(reviews)?,
            ),
            Action::Recovery { review, .. } => (
                "Review guarded recovery".into(),
                format!("Operation: {}\nAction: {}\n{}\nNo commits or hooks will be replayed.", review.operation_id, review.action, review.explanation),
                serde_json::to_string_pretty(review)?,
            ),
        };
        if summary.len() > 1024 * 1024 || content.len() > 32 * 1024 * 1024 {
            return Err(invalid("Review display limit exceeded"));
        }
        fn display_text(value: String) -> String {
            let mut result = String::with_capacity(value.len());
            for c in value.chars() {
                if (c.is_control() && c != '\n' && c != '\t')
                    || matches!(c,'\u{061c}'|'\u{200e}'|'\u{200f}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
                {
                    result.extend(c.escape_default());
                } else {
                    result.push(c);
                }
            }
            result
        }
        let display = ReviewDisplay {
            id: id.clone(),
            epoch: self.epoch,
            expires_at_ms: now_ms() + 600000,
            title,
            summary: display_text(summary),
            content: display_text(content),
        };
        self.pending = Some(Pending {
            id,
            epoch: self.epoch,
            created: Instant::now(),
            action,
            display: display.clone(),
            nonce: None,
        });
        Ok(display)
    }
    pub fn review(&self, id: &str) -> Result<ReviewDisplay> {
        let p = self.pending.as_ref().ok_or_else(stale)?;
        if p.id != id || p.epoch != self.epoch || p.created.elapsed() > Duration::from_secs(600) {
            return Err(stale());
        }
        Ok(p.display.clone())
    }
    /// Called by the privileged review window's native host, never by general IPC dispatch.
    pub fn begin_confirmation(&mut self, id: &str) -> Result<(Confirmation, ReviewDisplay)> {
        let display = self.review(id)?;
        let p = self.pending.as_mut().ok_or_else(stale)?;
        if p.nonce.is_some() {
            return Err(Error::with_kind(
                ErrorKind::Busy,
                "Confirmation already pending",
            ));
        }
        let nonce = Uuid::new_v4().to_string();
        p.nonce = Some(nonce.clone());
        Ok((
            Confirmation {
                id: id.into(),
                epoch: self.epoch,
                nonce,
            },
            display,
        ))
    }
    pub fn cancel_review(&mut self, id: &str) -> Result<()> {
        self.review(id)?;
        self.abandon_review()?;
        Ok(())
    }
    /// Only a Rust native confirmation callback may supply this capability.
    pub fn confirm(
        &mut self,
        capability: Confirmation,
        accepted: bool,
        cancellation: inspection::Cancellation,
    ) -> Result<serde_json::Value> {
        self.review(&capability.id)?;
        let p = self.pending.take().ok_or_else(stale)?;
        if p.epoch != capability.epoch || p.nonce.as_deref() != Some(&capability.nonce) {
            return Err(stale());
        }
        if !accepted {
            match &p.action {
                Action::Write(proposal) => self
                    .writes
                    .application_cancel_proposal(&proposal.operation_id)?,
                Action::Batch(record) => self.cancel_batch_record(&mut record.clone())?,
                _ => {}
            }
            self.writes.cancel_application_reviews();
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Native confirmation was declined",
            ));
        }
        let result = match p.action {
            Action::Batch(record) => self.execute_batch(record, cancellation)?,
            Action::Write(proposal) => {
                self.active(&proposal.workspace_id.0)?;
                self.entry(&proposal.workspace_id.0, &proposal.worktree_id.0)?;
                let approval = self.writes.approve(&proposal)?;
                serde_json::to_value(
                    self.writes
                        .execute_with_cancellation(&approval, cancellation)?,
                )?
            }
            Action::Trust { workspace, reviews } => {
                self.active(&workspace.0)?;
                let mut outcomes = Vec::new();
                for r in reviews {
                    let result = self
                        .entry(&workspace.0, &r.worktree_id.0)
                        .and_then(|_| self.writes.grant_trust(&r));
                    outcomes.push(match result {Ok(())=>json!({"worktree":r.worktree_id,"trusted":true}),Err(e)=>json!({"worktree":r.worktree_id,"trusted":false,"error":e.kind,"message":e.to_string()})});
                }
                json!({"trust_results":outcomes})
            }
            Action::Recovery { workspace, review } => {
                self.active(&workspace.0)?;
                self.writes
                    .application_operation_scope(&workspace, &review.operation_id)?;
                serde_json::to_value(self.writes.recover(&review)?)?
            }
        };
        Ok(result)
    }
}
