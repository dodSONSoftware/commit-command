use super::*;

use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub(super) fn sync(file: &File) -> Result<()> {
    file.sync_all()?;
    #[cfg(target_os = "macos")]
    {
        use std::os::fd::AsRawFd;
        if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(())
}
pub(super) fn sync_dir(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}
pub(super) fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    sync_dir(path.parent().ok_or_else(|| unsupported("Missing parent"))?)?;
    Ok(())
}
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    sync(&file)?;
    sync_dir(
        path.parent()
            .ok_or_else(|| unsupported("Missing artifact parent"))?,
    )
}
impl ControlledWrites {
    pub fn open(directory: &Path, mut options: Options) -> Result<Self> {
        if options.approval_lifetime > Duration::from_secs(600) {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Approval lifetime cannot exceed ten minutes",
            ));
        }
        if options.signer_home.is_none() {
            options.signer_home = std::env::var_os("GNUPGHOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(|home| PathBuf::from(home).join(".gnupg"))
                        .filter(|home| home.is_dir())
                });
        }
        let mut workspace = WorkspaceService::open(directory)?;
        let directory = workspace.data_directory().to_path_buf();
        let db = workspace.connection();
        let version: u32 = db.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version == 1 {
            let backup = directory.join(format!("state.pre-m2-{}.db", Uuid::new_v4()));
            db.backup(
                rusqlite::MAIN_DB,
                &backup,
                None::<fn(rusqlite::backup::Progress)>,
            )?;
            fs::set_permissions(&backup, fs::Permissions::from_mode(0o600))?;
            sync(&File::open(&backup)?)?;
            sync_dir(&directory)?;
            let tx = db.transaction()?;
            tx.execute_batch("CREATE TABLE write_trust(worktree_id TEXT PRIMARY KEY,review TEXT NOT NULL,revoked INTEGER NOT NULL DEFAULT 0,hook_policy TEXT NOT NULL DEFAULT 'execute');
                CREATE TABLE write_operations(id TEXT PRIMARY KEY,payload TEXT NOT NULL,approval_hash TEXT,approval_session TEXT);
                CREATE TABLE write_events(sequence INTEGER PRIMARY KEY AUTOINCREMENT,operation_id TEXT NOT NULL,phase TEXT NOT NULL,published INTEGER NOT NULL,new_oid TEXT,at_ms INTEGER NOT NULL);
                PRAGMA user_version=2;")?;
            tx.commit()?;
        }
        workspace.connection().execute_batch(
            "PRAGMA synchronous=FULL; PRAGMA fullfsync=ON; PRAGMA checkpoint_fullfsync=ON;",
        )?;
        let artifacts = directory.join("operations");
        if !artifacts.exists() {
            private_dir(&artifacts)?;
        }
        let metadata = fs::symlink_metadata(&artifacts)?;
        if !metadata.is_dir() || metadata.is_symlink() || metadata.permissions().mode() & 0o077 != 0
        {
            return Err(fail(
                ErrorKind::Storage,
                "Unsafe operation-artifact directory",
            ));
        }
        Ok(Self {
            workspace,
            options,
            session: Uuid::new_v4().to_string(),
            trust_reviews: BTreeMap::new(),
            recovery_reviews: BTreeMap::new(),
            #[cfg(feature = "write-test-support")]
            test_fault: None,
            #[cfg(all(feature = "write-test-support", feature = "application-services"))]
            import_fault: None,
            #[cfg(all(feature = "write-test-support", feature = "application-services"))]
            upstream_fault: None,
        })
    }
    pub(super) fn entry(
        &self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
    ) -> Result<crate::workspace::StoredWorktree> {
        let definition = self
            .workspace
            .list()
            .iter()
            .find(|w| &w.id == workspace)
            .ok_or_else(|| fail(ErrorKind::InvalidRequest, "Unknown workspace"))?;
        let entry = self
            .workspace
            .inventory(workspace)?
            .into_iter()
            .find(|e| &e.id == worktree)
            .ok_or_else(|| fail(ErrorKind::InvalidRequest, "Unknown worktree"))?;
        if !matches!(
            entry.availability,
            crate::workspace::Availability::Available
        ) || !entry
            .metadata
            .root
            .to_path()?
            .starts_with(definition.root.to_path()?)
        {
            return Err(fail(
                ErrorKind::Unavailable,
                "Worktree is unavailable or outside the current workspace",
            ));
        }
        Ok(entry)
    }
    pub fn inspect_trust(
        &mut self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
    ) -> Result<TrustReview> {
        let entry = self.entry(workspace, worktree)?;
        self.preflight_for_trust(&entry.metadata)?;
        let config = policy::effective(
            &entry.metadata,
            SigningPreference::Configured,
            &self.options,
        )?;
        let mut review = TrustReview {
            review_id: Uuid::new_v4().to_string(),
            repository_id: entry.repository_id,
            worktree_id: entry.id,
            metadata: entry.metadata.clone(),
            binding: policy::binding(&entry.metadata)?,
            hooks: config.hooks,
            signing: config.signing,
            config_hash: config.config_hash,
            config_origins: config.config_origins,
            signer_home: config.signer_home,
            digest: String::new(),
        };
        review.digest = digest(&serde_json::to_vec(&review)?);
        self.trust_reviews
            .insert(review.review_id.clone(), review.clone());
        Ok(review)
    }
    pub fn grant_trust(&mut self, review: &TrustReview) -> Result<()> {
        let expected = self
            .trust_reviews
            .remove(&review.review_id)
            .ok_or_else(|| {
                fail(
                    ErrorKind::StaleReview,
                    "Trust review is not owned by this session",
                )
            })?;
        if serde_json::to_vec(&expected)? != serde_json::to_vec(review)? {
            return Err(fail(ErrorKind::StaleReview, "Trust review was modified"));
        }
        self.preflight_for_trust(&review.metadata)?;
        let current = policy::effective(
            &review.metadata,
            SigningPreference::Configured,
            &self.options,
        )?;
        if current.config_hash != review.config_hash
            || current.hooks != review.hooks
            || current.signer_home != review.signer_home
            || current.signing != review.signing
            || policy::binding(&review.metadata)? != review.binding
        {
            return Err(fail(ErrorKind::StaleReview, "Trust context changed"));
        }
        self.workspace.connection().execute("INSERT INTO write_trust(worktree_id,review,revoked) VALUES(?1,?2,0) ON CONFLICT(worktree_id) DO UPDATE SET review=excluded.review,revoked=0",params![review.worktree_id.0,serde_json::to_string(review)?])?;
        Ok(())
    }
    pub fn revoke_trust(&mut self, id: &WorktreeId) -> Result<()> {
        self.workspace.connection().execute(
            "UPDATE write_trust SET revoked=1 WHERE worktree_id=?1",
            [&id.0],
        )?;
        Ok(())
    }
    pub fn remember_hook_policy(&mut self, id: &WorktreeId, policy: HookPolicy) -> Result<()> {
        let count = self.workspace.connection().execute(
            "UPDATE write_trust SET hook_policy=?2 WHERE worktree_id=?1 AND revoked=0",
            params![
                id.0,
                if policy == HookPolicy::Execute {
                    "execute"
                } else {
                    "bypass_all"
                }
            ],
        )?;
        if count != 1 {
            return Err(fail(
                ErrorKind::UntrustedRepository,
                "No current trust grant",
            ));
        }
        Ok(())
    }
    pub fn hook_policy(&mut self, id: &WorktreeId) -> Result<HookPolicy> {
        let value: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT hook_policy FROM write_trust WHERE worktree_id=?1 AND revoked=0",
                [&id.0],
                |row| row.get(0),
            )
            .optional()?;
        match value.as_deref() {
            Some("execute") => Ok(HookPolicy::Execute),
            Some("bypass_all") => Ok(HookPolicy::BypassAll),
            _ => Err(fail(
                ErrorKind::UntrustedRepository,
                "No current trust grant",
            )),
        }
    }
    pub(super) fn trusted(
        &mut self,
        id: &WorktreeId,
        meta: &Metadata,
        config: &EffectiveConfig,
    ) -> Result<()> {
        let row: Option<(String, bool)> = self
            .workspace
            .connection()
            .query_row(
                "SELECT review,revoked FROM write_trust WHERE worktree_id=?1",
                [&id.0],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (json, revoked) = row.ok_or_else(|| {
            fail(
                ErrorKind::UntrustedRepository,
                "Explicit worktree trust is required",
            )
        })?;
        let review: TrustReview = serde_json::from_str(&json)?;
        if revoked
            || policy::binding(meta)? != review.binding
            || review.config_hash != config.config_hash
            || review.hooks != config.hooks
            || review.signer_home != config.signer_home
        {
            self.revoke_trust(id)?;
            return Err(fail(
                ErrorKind::UntrustedRepository,
                "Trust was revoked or its identity/configuration changed",
            ));
        }
        // An explicitly unsigned proposal may omit a signer, but cannot replace it.
        if config.signing.is_some() && config.signing != review.signing {
            return Err(fail(
                ErrorKind::UntrustedRepository,
                "Signing executable/key policy changed",
            ));
        }
        Ok(())
    }
    pub(super) fn directory(&self, id: &str) -> Result<PathBuf> {
        Uuid::parse_str(id).map_err(|_| fail(ErrorKind::InvalidRequest, "Invalid operation ID"))?;
        Ok(self.workspace.data_directory().join("operations").join(id))
    }
    pub(super) fn load(&mut self, id: &str) -> Result<Operation> {
        self.directory(id)?;
        let value: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT payload FROM write_operations WHERE id=?1",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        serde_json::from_str(
            &value.ok_or_else(|| fail(ErrorKind::InvalidRequest, "Unknown operation"))?,
        )
        .map_err(Error::from)
    }
    pub(super) fn persist(&mut self, op: &Operation) -> Result<()> {
        let tx = self.workspace.connection().transaction()?;
        tx.execute("INSERT INTO write_operations(id,payload) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",params![op.proposal.operation_id,serde_json::to_string(op)?])?;
        tx.execute("INSERT INTO write_events(operation_id,phase,published,new_oid,at_ms) VALUES(?1,?2,?3,?4,?5)",params![op.proposal.operation_id,serde_json::to_string(&op.phase)?,op.published,op.new_head,now_ms() as i64])?;
        tx.commit()?;
        Ok(())
    }
    pub fn outcome(&mut self, id: &str) -> Result<Outcome> {
        Ok(self.load(id)?.outcome())
    }
    pub fn operations(&mut self) -> Result<Vec<Outcome>> {
        let mut statement = self
            .workspace
            .connection()
            .prepare("SELECT payload FROM write_operations ORDER BY rowid")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str::<Operation>(&row?)?.outcome()))
            .collect()
    }
    pub fn approve(&mut self, proposal: &Proposal) -> Result<Approval> {
        let mut op = self.load(&proposal.operation_id)?;
        if op.phase != Phase::Proposed
            || serde_json::to_vec(&op.proposal)? != serde_json::to_vec(proposal)?
        {
            return Err(fail(
                ErrorKind::StaleReview,
                "Proposal is stale or modified",
            ));
        }
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        self.workspace.connection().execute(
            "UPDATE write_operations SET approval_hash=?2,approval_session=?3 WHERE id=?1",
            params![
                proposal.operation_id,
                digest(token.as_bytes()),
                self.session
            ],
        )?;
        op.phase = Phase::Approved;
        self.persist(&op)?;
        Ok(Approval {
            operation: proposal.operation_id.clone(),
            token,
            session: self.session.clone(),
            issued: Instant::now(),
        })
    }
    pub(super) fn checkpoint(&mut self, op: &mut Operation, phase: Phase) -> Result<()> {
        op.phase = phase;
        self.persist(op)?;
        #[cfg(feature = "write-test-support")]
        if self
            .test_fault
            .as_ref()
            .is_some_and(|fault| fault.at == phase)
        {
            let root = op.metadata.root.to_path()?;
            let temporary = fs::canonicalize(std::env::temp_dir())?;
            if !root.starts_with(&temporary)
                || !root.parent().is_some_and(|p| {
                    p.file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with("gitfleet-"))
                })
                || !root.join(".gitfleet-m2-fixture").is_file()
            {
                return Err(fail(
                    ErrorKind::InvalidRequest,
                    "Fault injection requires a disposable fixture",
                ));
            }
            match self.test_fault.take().expect("checked fault").kind {
                TestFaultKind::Sigkill => {
                    if unsafe { libc::kill(libc::getpid(), libc::SIGKILL) } != 0 {
                        return Err(fail(
                            ErrorKind::Storage,
                            "Could not deliver fixture SIGKILL",
                        ));
                    }
                    // Delivery can be asynchronous; never substitute an ordinary exit.
                    loop {
                        std::thread::park();
                    }
                }
                TestFaultKind::StorageFailure => {
                    return Err(fail(
                        ErrorKind::Storage,
                        "Injected durable-boundary storage failure",
                    ))
                }
                TestFaultKind::CorruptCandidate => fs::write(
                    self.directory(&op.proposal.operation_id)?
                        .join("reconciled.index"),
                    b"corrupt",
                )?,
            }
        }
        Ok(())
    }
    #[cfg(feature = "write-test-support")]
    pub fn set_test_fault(&mut self, fault: TestFault) {
        self.test_fault = Some(fault);
    }
}

impl ControlledWrites {
    pub(super) fn ensure_fetch_import_resolved(
        &mut self,
        common: &str,
        except: Option<&str>,
    ) -> Result<()> {
        let pending:bool=self.workspace.connection().query_row("SELECT EXISTS(SELECT 1 FROM state WHERE key LIKE 'fetch-import:%' AND json_extract(value,'$.value.plan.metadata.common_binding')=?1 AND (?2 IS NULL OR json_extract(value,'$.value.plan.id')<>?2) AND json_extract(value,'$.value.phase') IN ('importing_objects','publication_pending','reference_prepared','reference_updated','uncertain'))",params![common,except],|r|r.get(0))?;
        if pending {
            return Err(fail(
                ErrorKind::Busy,
                "Inspect interrupted fetch import before another repository write",
            ));
        }
        Ok(())
    }
}

impl ControlledWrites {
    pub(super) fn ensure_upstream_resolved(
        &mut self,
        common: &str,
        except: Option<&str>,
    ) -> Result<()> {
        let pending:bool=self.workspace.connection().query_row("SELECT EXISTS(SELECT 1 FROM state WHERE key LIKE 'upstream:%' AND json_extract(value,'$.value.plan.metadata.common_binding')=?1 AND (?2 IS NULL OR json_extract(value,'$.value.plan.id')<>?2) AND json_extract(value,'$.value.phase') IN ('lock_pending','lock_created','lock_ready','publication_pending','config_published','uncertain'))",params![common,except],|r|r.get(0))?;
        if pending {
            return Err(fail(
                ErrorKind::Busy,
                "Inspect interrupted upstream setup before another repository write",
            ));
        }
        Ok(())
    }
}
