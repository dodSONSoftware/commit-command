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
    pub(super) fn set_review(&mut self, action: Action) -> Result<ReviewDisplay> {
        let id = Uuid::new_v4().to_string();
        let (title, summary, content) = match &action {
            Action::Write(p) => {
                let entry = self.entry(&p.workspace_id.0, &p.worktree_id.0)?;
                let mut plan = serde_json::to_value(p)?;
                if let Some(fields) = plan.as_object_mut() { fields.remove("patch"); }
                let mut detail = serde_json::to_string_pretty(&plan)?;
                detail.push_str(&format!("\nCanonical worktree: {:?}\n", entry.metadata.root));
                detail.push_str("\nExact message (JSON escaped):\n");
                detail.push_str(&serde_json::to_string(&String::from_utf8_lossy(&p.message))?);
                detail.push_str("\n\nProposed patch (control characters escaped except line breaks):\n");
                for c in String::from_utf8_lossy(&p.patch).chars() {
                    if c == '\n' || c == '\t' || !c.is_control() { detail.push(c); }
                    else { detail.extend(c.escape_default()); }
                }
                let kind = match &p.request { WriteRequest::Commit { .. } => "COMMIT", WriteRequest::Stage { .. } => "STAGE", WriteRequest::Unstage { .. } => "UNSTAGE" };
                let subject = String::from_utf8_lossy(&p.message).lines().next().unwrap_or("").chars().take(160).collect::<String>();
                let summary = format!("{kind}\nWorktree: {:?}\nRepository: {}\nBranch: {}\nTree: {}\nMessage subject: {}\nHooks: {:?}\nSigner: {}\nReview digest: {}\nFull content is in the authoritative review window.", entry.metadata.root, p.repository_id.0, p.branch, p.approved_tree, serde_json::to_string(&subject)?, p.hooks, p.signing.as_ref().map(|s| s.fingerprint.as_str()).unwrap_or("UNSIGNED"), p.review_digest);
                ("Review controlled write".into(), summary, detail)
            }
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
            if let Action::Write(proposal) = &p.action {
                self.writes
                    .application_cancel_proposal(&proposal.operation_id)?;
            }
            self.writes.cancel_application_reviews();
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Native confirmation was declined",
            ));
        }
        let result = match p.action {
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
