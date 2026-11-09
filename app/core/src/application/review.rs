use super::*;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReviewDisplay {
    pub id: String,
    pub epoch: u64,
    pub expires_at_ms: u64,
    pub title: String,
    pub summary: String,
    pub content: String,
    /// Set only by Core for approved provider reads; defaults fail closed.
    #[serde(default)]
    pub allow_in_read_only_build: bool,
    /// Display correlation only, never a confirmation capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_request_fingerprint: Option<String>,
}
pub(super) struct AiSourceBinding {
    pub settings_fingerprint: String,
    pub workspace: String,
    pub worktree: String,
    pub spec: crate::ai::capture::Spec,
    pub captured: crate::ai::capture::Captured,
}
pub(super) enum Action {
    Ai {
        plan: Box<crate::ai::provider::Plan>,
        limits: crate::ai::jobs::Limits,
        settings_fingerprint: String,
        source: Option<Box<AiSourceBinding>>,
    },
    Version(Box<crate::actions::version::VersionPlan>),
    VersionRecovery(Box<crate::actions::version::VersionRecovery>),
    Authentication(Box<super::authentication::SettingsPlan>),
    Trust {
        workspace: WorkspaceId,
        reviews: Vec<TrustReview>,
    },
    Write(Box<Proposal>),
    PushCheck(Box<crate::remote::push_check::PushCheckPlan>),
    PushPublication(Box<crate::remote::push_publication::PushPublicationPlan>),
    PushPublicationRecovery(Box<crate::remote::push_publication::PushPublicationRecovery>),
    Batch(BatchRecord),
    RemoteObservation(Box<crate::remote::observation::ObservationPlan>),
    FetchCapture(Box<crate::remote::fetch::FetchCapturePlan>),
    FetchImport(Box<crate::remote::fetch::FetchImportPlan>),
    FetchImportRecovery(Box<crate::remote::fetch::FetchImportRecovery>),
    UpstreamSetup(Box<crate::remote::upstream::UpstreamPlan>),
    UpstreamRecovery(Box<crate::remote::upstream::UpstreamRecovery>),
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
        let allow_in_read_only_build = matches!(&action, Action::Ai { .. });
        let ai_request_fingerprint = match &action {
            Action::Ai { plan, .. } => Some(plan.review().fingerprint),
            _ => None,
        };
        let (title, summary, content) = match &action {
            Action::Ai { plan, .. } => {
                let review = plan.review();
                let scope=if review.source.is_some() {"This transfers the exact filtered repository content below. Inspect coverage and omissions. Incomplete analysis may suggest text only; no automatic commit is authorized."}else{"No repository content is captured by this diagnostic."};
                let mut summary = format!("Approve one optional AI provider request only.\nPurpose: {:?}\nExact destination: {}\nModel: {}\nAuthentication: {}\nCredential connection: {}\nRequest fingerprint: {}\nPayload bytes: {}\n{}\n{scope} No Git, Action or repository-write authority. Selected credential use is limited to this saved connection and exact endpoint for this request. Redirects are refused; no model fallback. Cancellation discards the result; restart never resumes it.", review.purpose, review.route, review.model.as_deref().unwrap_or("Model discovery"), match review.authentication { crate::ai::credentials::Authentication::None=>"No credentials", crate::ai::credentials::Authentication::StoredBearer=>"Stored bearer token" }, review.credential.as_ref().map(|b| b.connection.as_str()).unwrap_or("None"), review.fingerprint, review.request_bytes, if review.unencrypted { "HTTP IS UNENCRYPTED." } else { "HTTPS certificate and hostname verification required." });
                if review.purpose == crate::ai::provider::Purpose::EditorTest {
                    summary.push_str("\nUnsaved editor test only: settings and assignments are not saved. Results cannot authorize an Action or automatic commit.");
                }
                let payload = plan.payload().map(|p| String::from_utf8_lossy(p).into_owned()).unwrap_or_else(|| "No request body".into());
                (if review.purpose == crate::ai::provider::Purpose::EditorTest {"Review AI source transfer and editor test"}else if review.source.is_some() {"Review AI source transfer and message suggestions"}else{"Review optional AI provider diagnostic"}.into(), summary, format!("{}\n\nExact request payload (source text is data)\n{}", serde_json::to_string_pretty(&review)?, payload))
            },
            Action::Version(plan)=>("Review exact version Action".into(),format!("Approve this typed definition and one invocation only.\nWorktree: {}\nFile: {}\nOld: {}\nNew: {}\nIncrement: {:?}; prerelease: {:?}\nReplace only the exact reviewed regular file, preserving its mode and newline. The previous file is retained in private recovery storage. No hooks, AI, network, index, refs or commit. Staging and committing require separate fresh reviews. Context drift stops execution; a last-instant external edit is retained and reported for reconciliation. Interrupted replacement is inspected without replay or double increment.\nDigest: {}",plan.metadata.root.display(),plan.path.display(),plan.version.old,plan.version.new,plan.version.increment,plan.version.prerelease,plan.digest),serde_json::to_string_pretty(plan)?),
            Action::VersionRecovery(review)=>("Review observed version Action outcome".into(),format!("Invocation: {}\nFile: {}\nObserved: {:?}\nRecord this observed state only. No file, index, ref, hook, AI or network operation is replayed. Conflicting external content is preserved. The previous file or unfinished candidate is retained in private recovery storage; new mutation/staging requires fresh review.",review.invocation.plan.id,review.invocation.plan.path.display(),review.observed),serde_json::to_string_pretty(review)?),
            Action::Authentication(plan) => (
                "Review authentication settings".into(),
                format!("Requested change:\n{}\nSave the exact nonsecret profiles and remote assignments below. Selected helpers run unsandboxed as your user and may prompt during separately approved operations. Saving settings does not call a credential helper, contact a server, trust a repository or approve any network operation. Unknown or changed SSH host keys stop execution; verify and update known_hosts externally. Any settings revision invalidates outstanding authenticated network reviews.",plan.description),
                serde_json::to_string_pretty(plan)?,
            ),
            Action::PushPublication(plan) => (
                "Review exact fast-forward push".into(),
                format!("{}PUSH PUBLICATION\nWorktree: {}\nSource: {} at {}\nDestination: {} · {}\nExpected old commit: {}\nOutgoing commits: {}\nHook policy: {:?}\nThis new approval runs the reviewed pre-push check, exports only the approved committed object closure, checks the exact destination, and attempts one expected-old guarded fast-forward update. Core preserves managed files, refs, config and upstreams; reviewed hooks can have their own effects. No force update, destination creation, implicit child push or automatic retry is authorized. At most 10,000 objects / 64 MiB uncompressed; publication may refuse if these bounds are exceeded. Hooks run unsandboxed as your user. Cancellation or process loss after transport begins may leave publication unconfirmed. Approval includes destination observations immediately before and after this push; authenticated helper/agent policy is the bound policy in the exact plan.\nDigest: {}",plan.batch.as_ref().map(|b|format!("Guided queue {}: this approval covers only this item. Remaining items require fresh evidence and separate native approvals.\n",b.id)).unwrap_or_default(),plan.check.transport.metadata.root.display(),plan.check.preview.source_ref,plan.check.preview.source_oid.hex,plan.check.preview.destination,plan.check.preview.destination_ref,plan.check.preview.observed_oid.hex,plan.check.preview.outgoing_count,plan.check.hooks,plan.digest),
                serde_json::to_string_pretty(plan)?,
            ),
            Action::PushPublicationRecovery(review) => (
                "Review observed push outcome".into(),
                format!("Operation: {}\nDestination: {} · {}\nSeparately approved observation: {}\nObserved state: {:?}\nRecord only this past observation of the destination. It does not prove which actor published a commit. No network, push, hook, ref change or automatic retry is authorized by this recovery approval.",review.operation.plan.id,review.operation.plan.check.preview.destination,review.operation.plan.check.preview.destination_ref,review.observation.plan.id,review.remote_state),
                serde_json::to_string_pretty(review)?,
            ),
            Action::PushCheck(plan) => (
                "Review pre-push hook check — no publication".into(),
                format!("Check {} at {} for {} on {}.\nExpected remote commit: {}\nHook policy: {:?}\nAll required child commits must already be reachable from fresh, separately approved observations. Conditional child order is not success.\nApproval runs only the exact reviewed pre-push hook (unless explicitly bypassed) and revalidates context. Hooks run unsandboxed as your user and can modify files or contact servers. Raw output is not retained. Core starts no push transport and changes no refs or upstreams. A completed check grants no publication authority and cannot be reused or replayed. Future push execution requires a new exact review.\nReview digest: {}",plan.preview.source_ref,plan.preview.source_oid.hex,plan.preview.destination_ref,plan.preview.destination,plan.preview.observed_oid.hex,plan.hooks,plan.digest),
                serde_json::to_string_pretty(plan)?,
            ),
            Action::Write(p) => self.describe_write(p)?,
            Action::Batch(record) => self.describe_batch(record)?,
            Action::UpstreamSetup(plan)=>(
                "Review missing upstream setup".into(),
                format!("Set upstream for {} at {}.\nRemote: {} · {}\nMerge ref: {}\nAdd missing branch remote/merge settings to shared config. Existing branch settings are refused.\nConfig before SHA-256: {}\nConfig candidate SHA-256: {}\nThis contacts no server and changes no objects, refs, index or working files. All linked worktrees share this config. Existing trust becomes stale and requires fresh review. No push or automatic retry is authorized.",plan.source_ref,plan.source_oid.hex,plan.transport.remote,plan.transport.destination,plan.transport.reference,plan.expected_config.digest,plan.candidate_digest),
                serde_json::to_string_pretty(plan)?,
            ),
            Action::UpstreamRecovery(review)=>("Review upstream configuration recovery".into(),format!("Operation: {}\nAction: {}\n{}",review.operation.plan.transport.id,review.action,review.explanation),serde_json::to_string_pretty(review)?),
            Action::FetchImport(plan)=>(
                "Review captured branch import".into(),
                format!("Import verified captured objects and update {}.\nExpected old value: {}\nNew captured commit: {}\nBundle SHA-256: {} ({} bytes)\nThis offline operation adds objects and writes this exact tracking ref and its reflog. It does not merge, change HEAD, index, working files, tags, upstreams or FETCH_HEAD. Hooks and network are disabled. Interrupted publication requires inspection, never automatic replay.",plan.tracking_ref,plan.expected_old.as_ref().map(|v|v.hex.as_str()).unwrap_or("absent"),plan.expected_oid.hex,plan.bundle_digest,plan.bundle_bytes),
                serde_json::to_string_pretty(plan)?,
            ),
            Action::FetchImportRecovery(review)=>("Review captured branch recovery".into(),format!("Operation: {}\nRef: {}\nAction: {}\n{}",review.operation.plan.transport.id,review.operation.plan.tracking_ref,review.action,review.explanation),serde_json::to_string_pretty(review)?),
            Action::FetchCapture(plan) => (
                "Review branch download for fetch".into(),
                format!("Download {} on remote {} from {}.\nExpected observed commit: {}\nThe downloaded branch must still match that exact commit.\nObjects stay in isolated app storage. This does not update this checkout, tracking refs, index, working files, tags or upstreams. Import requires a separate review.\n{}\nManaged credential helpers, proxy, SSH commands, hooks and submodule recursion are disabled.\nLimits: 10,000 objects, 64 MiB uncompressed/bundle, 128 MiB polled artifact budget and the bounded process deadline.",plan.transport.reference,plan.transport.remote,plan.transport.destination,plan.expected_oid.hex,transport_summary(&plan.transport)),
                serde_json::to_string_pretty(plan)?,
            ),
            Action::RemoteObservation(plan) => (
                "Review remote branch observation".into(),
                format!("Contact {} to observe {} on remote {}.\nNo commits, working files, index, refs or upstreams will be changed.\n{}\nManaged transport configuration, proxy and SSH commands are not executed.\nLocal transport is restricted to host-configured disposable fixtures.", plan.destination, plan.reference, plan.remote,transport_summary(plan)),
                serde_json::to_string_pretty(plan)?,
            ),
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
            allow_in_read_only_build,
            ai_request_fingerprint,
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
                Action::Version(plan) => self.writes.cancel_version(&plan.workspace, &plan.id)?,
                Action::Write(proposal) => self
                    .writes
                    .application_cancel_proposal(&proposal.operation_id)?,
                Action::Batch(record) => self.cancel_batch_record(&mut record.clone())?,
                Action::RemoteObservation(plan) => self.writes.cancel_observation(&plan.id)?,
                Action::PushCheck(plan) => self.writes.cancel_push_check(&plan.id)?,
                Action::PushPublication(plan) => self.writes.cancel_publication(&plan.id)?,
                Action::UpstreamSetup(plan) => self.writes.cancel_upstream(&plan.transport.id)?,
                Action::FetchImport(plan) => self.writes.cancel_import(&plan.transport.id)?,
                Action::FetchCapture(plan) => self.writes.cancel_capture(&plan.transport.id)?,
                _ => {}
            }
            self.writes.cancel_application_reviews();
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Native confirmation was declined",
            ));
        }
        let result = match p.action {
            Action::Ai {
                plan,
                limits,
                settings_fingerprint,
                source,
            } => {
                if crate::ai::store::load(self.writes.workspace.data_directory())?.fingerprint
                    != settings_fingerprint
                {
                    return Err(stale());
                }
                if cancellation.is_cancelled() {
                    return Err(Error::with_kind(
                        ErrorKind::Cancelled,
                        "AI diagnostic cancelled",
                    ));
                }
                if let Some(source) = source {
                    let entry = self.entry(&source.workspace, &source.worktree)?;
                    let mut options = self.read_options.clone();
                    options.cancellation = cancellation.clone();
                    crate::ai::capture::revalidate(
                        &entry.metadata.root.to_path()?,
                        &source.spec,
                        &source.captured,
                        &options,
                    )?;
                }
                let key = crate::domain::digest(plan.review().endpoint.as_bytes());
                let job = self.ai_jobs.start(
                    (*plan).approve(),
                    self.ai_secret_store.clone(),
                    key,
                    limits,
                    cancellation,
                )?;
                json!({ "ai_job": job })
            }
            Action::Version(plan) => {
                self.active(&plan.workspace.0)?;
                self.entry(&plan.workspace.0, &plan.worktree.0)?;
                serde_json::to_value(self.writes.execute_version(&plan, cancellation)?)?
            }
            Action::VersionRecovery(review) => {
                self.active(&review.invocation.plan.workspace.0)?;
                if cancellation.is_cancelled() {
                    return Err(Error::with_kind(
                        ErrorKind::Cancelled,
                        "Version recovery cancelled",
                    ));
                }
                serde_json::to_value(self.writes.recover_version(&review)?)?
            }
            Action::Authentication(plan) => self.apply_authentication(*plan, cancellation)?,
            Action::PushPublication(plan) => {
                self.active(&plan.check.workspace.0)?;
                let entries =
                    self.push_check_entries(&plan.check.workspace.0, &plan.check.selection)?;
                serde_json::to_value(self.writes.execute_publication(
                    &plan,
                    &entries,
                    cancellation,
                )?)?
            }
            Action::PushPublicationRecovery(review) => {
                self.active(&review.operation.plan.check.workspace.0)?;
                if cancellation.is_cancelled() {
                    return Err(Error::with_kind(ErrorKind::Cancelled, "Recovery cancelled"));
                }
                serde_json::to_value(self.writes.recover_publication(&review)?)?
            }
            Action::PushCheck(plan) => {
                self.active(&plan.workspace.0)?;
                let entries = self.push_check_entries(&plan.workspace.0, &plan.selection)?;
                serde_json::to_value(self.writes.execute_push_check(
                    &plan,
                    &entries,
                    cancellation,
                )?)?
            }
            Action::UpstreamSetup(plan) => {
                self.active(&plan.transport.workspace.0)?;
                self.entry(&plan.transport.workspace.0, &plan.transport.worktree.0)?;
                serde_json::to_value(self.writes.execute_upstream(&plan, cancellation)?)?
            }
            Action::UpstreamRecovery(review) => {
                self.active(&review.operation.plan.transport.workspace.0)?;
                self.entry(
                    &review.operation.plan.transport.workspace.0,
                    &review.operation.plan.transport.worktree.0,
                )?;
                if cancellation.is_cancelled() {
                    return Err(Error::with_kind(ErrorKind::Cancelled, "Recovery cancelled"));
                }
                serde_json::to_value(self.writes.recover_upstream(&review)?)?
            }
            Action::FetchImport(plan) => {
                self.active(&plan.transport.workspace.0)?;
                self.entry(&plan.transport.workspace.0, &plan.transport.worktree.0)?;
                serde_json::to_value(self.writes.execute_import(&plan, cancellation)?)?
            }
            Action::FetchImportRecovery(review) => {
                self.active(&review.operation.plan.transport.workspace.0)?;
                self.entry(
                    &review.operation.plan.transport.workspace.0,
                    &review.operation.plan.transport.worktree.0,
                )?;
                if cancellation.is_cancelled() {
                    return Err(Error::with_kind(ErrorKind::Cancelled, "Recovery cancelled"));
                }
                serde_json::to_value(self.writes.recover_import(&review)?)?
            }
            Action::FetchCapture(plan) => {
                self.active(&plan.transport.workspace.0)?;
                self.entry(&plan.transport.workspace.0, &plan.transport.worktree.0)?;
                serde_json::to_value(self.writes.execute_capture(&plan, cancellation)?)?
            }
            Action::RemoteObservation(plan) => {
                self.active(&plan.workspace.0)?;
                self.entry(&plan.workspace.0, &plan.worktree.0)?;
                serde_json::to_value(self.writes.execute_observation(&plan, cancellation)?)?
            }
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

fn transport_summary(plan: &crate::remote::observation::ObservationPlan) -> String {
    let profile = plan
        .authentication_profile
        .as_ref()
        .map(|p| {
            format!(
                "Authentication profile: {} · {} · settings revision {}\n",
                p.name, p.id, p.revision
            )
        })
        .unwrap_or_default();
    format!("{profile}{}", transport_policy_summary(plan))
}
fn transport_policy_summary(plan: &crate::remote::observation::ObservationPlan) -> String {
    if plan.transport == crate::remote::Transport::Ssh {
        let bindings = plan
            .tools
            .iter()
            .filter(|b| b.role.starts_with("ssh_"))
            .map(|b| format!("{}: {:?} · {}", b.role, b.path, b.fingerprint))
            .collect::<Vec<_>>()
            .join("\n");
        format!("SSH authenticates only the reviewed public identity through the selected agent. Strict known-hosts verification is required; unknown or changed keys stop execution. Verify and update known_hosts externally; no password/keychain fallback, private-key loading, forwarding or host-key writes.\n{bindings}")
    } else if let Some(scope) = &plan.authentication {
        let helper = plan
            .tools
            .iter()
            .find(|t| t.role == "credential_helper")
            .map(|binding| {
                let path = binding
                    .path
                    .to_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "Unavailable path".into());
                format!("{path} · SHA-256 {}", binding.fingerprint)
            })
            .unwrap_or_else(|| "Unavailable helper binding".into());
        let home = plan
            .tools
            .iter()
            .find(|t| t.role == "credential_home")
            .and_then(|t| t.path.to_path().ok())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Unavailable".into());
        format!("HTTPS uses only Basic authentication over verified TLS and rejects redirects. Use account {} only for https://{}/{}.\nApproved helper: {}\nHelper HOME: {} (Git transport stays isolated)\nThe trusted helper runs unsandboxed as your user and may access its credential store. Only get is sent; Git store/erase are discarded. No credential is shown or saved in GitFleet journals.",scope.account,scope.host,scope.path,helper,home)
    } else {
        "HTTPS verifies certificates and hostnames against the reviewed trust source and rejects redirects. No credentials or credential helpers are used.".into()
    }
}
