//! Reviewed network reads use a new transport repository with no managed configuration.
use super::*;
use crate::domain::ObjectId;
use crate::remote::{self, observation::*, Transport};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct StoredObservation {
    version: u32,
    session: String,
    value: Observation,
}
fn stale() -> Error {
    fail(ErrorKind::StaleReview, "Remote observation review is stale")
}
fn tool(role: &str, path: &Path) -> Result<ToolBinding> {
    let bound_path = if role == "https_helper" {
        path.to_path_buf()
    } else {
        fs::canonicalize(path)?
    };
    Ok(ToolBinding {
        role: role.into(),
        path: NativePath::from_path(&bound_path),
        fingerprint: adapter::program_fingerprint(path)?,
    })
}
fn tool_path<'a>(plan: &'a ObservationPlan, role: &str) -> Result<&'a NativePath> {
    plan.tools
        .iter()
        .find(|t| t.role == role)
        .map(|t| &t.path)
        .ok_or_else(stale)
}
fn toolchain(
    options: &ReadOptions,
    ca: Option<&Path>,
    transport: &Transport,
) -> Result<Vec<ToolBinding>> {
    let git = adapter::find_program("git")
        .map_err(|_| fail(ErrorKind::Unavailable, "Git executable is unavailable"))?;
    let mut command = adapter::clean(&git);
    command.arg("--exec-path");
    let output = process::bounded_input(command, options, None)?;
    if !output.status.success() {
        return Err(fail(
            ErrorKind::Unavailable,
            "Git transport directory is unavailable",
        ));
    }
    let directory = fs::canonicalize(inspection::path_output(output.stdout)?)?;
    let mut tools = vec![tool("git", &git)?];
    if *transport == Transport::Https {
        tools.push(tool("https_helper", &directory.join("git-remote-https"))?);
    }
    // Bind the executable search directory independently of the helper's canonical file path.
    tools.push(ToolBinding {
        role: "exec_directory".into(),
        path: NativePath::from_path(&directory),
        fingerprint: inspection::fs_identity(&directory)?,
    });
    if let Some(ca) = ca {
        tools.push(tool("certificate_authority", ca)?);
    }
    Ok(tools)
}
fn validate_tools(plan: &ObservationPlan) -> Result<()> {
    for binding in &plan.tools {
        let path = binding.path.to_path()?;
        let actual = if binding.role == "ssh_agent_socket" {
            super::ssh_transport::socket_binding(&path)?.fingerprint
        } else if binding.role == "exec_directory" {
            inspection::fs_identity(&path)?
        } else {
            adapter::program_fingerprint(&path)?
        };
        if actual != binding.fingerprint {
            return Err(stale());
        }
    }
    Ok(())
}
pub(super) fn command(plan: &ObservationPlan, cwd: &Path, home: &Path) -> Result<Command> {
    let mut command = Command::new(tool_path(plan, "git")?.to_path()?);
    let exec = tool_path(plan, "exec_directory")?.to_path()?;
    let path = std::env::join_paths([exec.as_path(), Path::new("/usr/bin"), Path::new("/bin")])
        .map_err(|_| unsupported("Transport executable path cannot be represented"))?;
    command
        .env_clear()
        .current_dir(cwd)
        .env("PATH", path)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env("CURL_HOME", home)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .env("GIT_EXEC_PATH", exec)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_ASKPASS", "")
        .env(
            "GIT_ALLOW_PROTOCOL",
            match plan.transport {
                Transport::Https => "https",
                Transport::Ssh => "ssh",
                _ => "file",
            },
        )
        .args([
            "--no-pager",
            "--literal-pathspecs",
            "-c",
            "core.hooksPath=",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "init.templateDir=",
            "-c",
            "maintenance.auto=false",
            "-c",
            "gc.auto=0",
            "-c",
            "http.sslVerify=true",
            "-c",
            "http.followRedirects=false",
            "-c",
            "http.emptyAuth=false",
            "-c",
            "http.proactiveAuth=none",
            "-c",
            "http.proxy=",
            "-c",
            "credential.helper=",
            "-c",
            "core.askPass=",
            "-c",
            "http.extraHeader=",
            "-c",
            "http.cookieFile=",
            "-c",
            "http.saveCookies=false",
            "-c",
            "submodule.recurse=false",
        ]);
    if let Some(scope) = &plan.authentication {
        command
            .args([
                "-c",
                "http.proactiveAuth=basic",
                "-c",
                "credential.useHttpPath=true",
                "-c",
            ])
            .arg(format!("credential.username={}", scope.account))
            .arg("-c")
            .arg(format!(
                "credential.helper={}",
                crate::credentials::helper_command(scope, &plan.tools)?
            ));
    }
    if plan.transport == Transport::Ssh {
        command
            .env(
                "GIT_SSH_COMMAND",
                super::ssh_transport::command(&plan.tools, home)?,
            )
            .env("GIT_SSH_VARIANT", "ssh");
    }
    if let Some(ca) = plan
        .tools
        .iter()
        .find(|t| t.role == "certificate_authority")
    {
        let mut argument = std::ffi::OsString::from("http.sslCAInfo=");
        argument.push(ca.path.to_path()?);
        command.arg("-c").arg(argument);
    }
    Ok(command)
}
fn local_target(root: &Path, url: &str, policy: &remote::TransportOptions) -> Result<PathBuf> {
    let allowed = policy.local_fixture_root.as_ref().ok_or_else(|| {
        unsupported(
            "Local transport is restricted to an explicitly configured disposable fixture root",
        )
    })?;
    let candidate = if url.starts_with("file://") {
        url::Url::parse(url)
            .map_err(|_| unsupported("Invalid local fixture URL"))?
            .to_file_path()
            .map_err(|_| unsupported("Invalid local fixture path"))?
    } else {
        let path = Path::new(url);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    };
    if candidate
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
        || !candidate.starts_with(allowed)
    {
        return Err(unsupported(
            "Local transport destination is outside the approved fixture root",
        ));
    }
    let target = fs::canonicalize(&candidate)?;
    if !target.starts_with(fs::canonicalize(allowed)?) || !target.is_dir() {
        return Err(unsupported("Local fixture destination binding is invalid"));
    }
    Ok(target)
}
impl ControlledWrites {
    fn persist_observation(&mut self, stored: &StoredObservation) -> Result<()> {
        let payload = serde_json::to_string(stored)?;
        if payload.len() > 1024 * 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Observation record exceeds 1 MiB",
            ));
        }
        self.workspace.connection().execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES(?1,?2)",
            params![
                format!("remote-observation:{}", stored.value.plan.id),
                payload
            ],
        )?;
        Ok(())
    }
    fn observation(&mut self, id: &str) -> Result<StoredObservation> {
        Uuid::parse_str(id).map_err(|_| stale())?;
        let payload: Option<String> = self
            .workspace
            .connection()
            .query_row(
                "SELECT value FROM state WHERE key=?1",
                [format!("remote-observation:{id}")],
                |r| r.get(0),
            )
            .optional()?;
        let stored: StoredObservation = serde_json::from_str(&payload.ok_or_else(stale)?)?;
        if stored.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported remote observation version",
            ));
        }
        Ok(stored)
    }
    pub(crate) fn propose_observation(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        remote: &str,
        reference: &str,
        options: &ReadOptions,
    ) -> Result<ObservationPlan> {
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = self
            .prepare_transport_plan(workspace, entry, remote, reference, options)
            .and_then(|plan| {
                self.persist_observation(&StoredObservation {
                    version: 1,
                    session: self.session.clone(),
                    value: Observation {
                        plan: plan.clone(),
                        phase: ObservationPhase::Proposed,
                        oid: None,
                        observed_at_ms: None,
                        issue: None,
                    },
                })?;
                Ok(plan)
            });
        self.options.process = previous;
        result
    }
    pub(super) fn prepare_transport_plan(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        remote: &str,
        reference: &str,
        options: &ReadOptions,
    ) -> Result<ObservationPlan> {
        if reference.len() > 1024 || !reference.starts_with("refs/heads/") {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Choose one full refs/heads/ reference",
            ));
        }
        let root = entry.metadata.root.to_path()?;
        inspection::read(&root, options, |s| {
            s.git(&["check-ref-format", reference]).map(|_| ())
        })?;
        policy::preflight(&entry.metadata, &self.options)?;
        let config =
            policy::effective(&entry.metadata, SigningPreference::Unsigned, &self.options)?;
        self.trusted(&entry.id, &entry.metadata, &config)?;
        let resolved = remote::resolve_for_observation(
            &root,
            self.options.global_config.as_deref(),
            remote,
            options,
        )?;
        let local = if resolved.transport == Transport::Local {
            Some(local_target(&root, &resolved.url, &self.options.remote)?)
        } else {
            None
        };
        let local_binding = local.as_deref().map(inspection::fs_identity).transpose()?;
        if resolved.transport == Transport::Ssh {
            super::ssh_transport::validate_destination(&resolved.url)?;
        }
        let mut tools = toolchain(
            options,
            self.options.remote.ca_file.as_deref(),
            &resolved.transport,
        )?;
        if resolved.transport == Transport::Ssh {
            tools.extend(super::ssh_transport::bindings(
                self.options.remote.ssh.as_ref(),
            )?);
        }
        let authentication = if resolved.transport == Transport::Https {
            if let Some(policy) = self.options.remote.https_credentials.as_ref() {
                tools.extend(crate::credentials::bindings(policy)?);
                Some(crate::credentials::scope(&resolved.url, &policy.account)?)
            } else {
                None
            }
        } else {
            None
        };
        let plan = ObservationPlan {
            id: Uuid::new_v4().to_string(),
            workspace: workspace.clone(),
            worktree: entry.id.clone(),
            metadata: entry.metadata.clone(),
            remote: remote.into(),
            reference: reference.into(),
            destination: resolved
                .inspection
                .remotes
                .iter()
                .find(|r| r.name == remote)
                .ok_or_else(stale)?
                .fetch[0]
                .display
                .clone(),
            transport: resolved.transport.clone(),
            config_digest: resolved.inspection.config_digest,
            generation: resolved.inspection.generation,
            created_at_ms: now_ms(),
            tools,
            local_target: local.as_deref().map(NativePath::from_path),
            local_binding,
            authentication,
        };
        Ok(plan)
    }
    pub(crate) fn cancel_observation(&mut self, id: &str) -> Result<()> {
        let mut stored = self.observation(id)?;
        if stored.value.phase == ObservationPhase::Proposed {
            stored.value.phase = ObservationPhase::Cancelled;
            self.persist_observation(&stored)?;
        }
        Ok(())
    }
    pub(crate) fn execute_observation(
        &mut self,
        plan: &ObservationPlan,
        cancellation: inspection::Cancellation,
    ) -> Result<Observation> {
        let previous =
            std::mem::replace(&mut self.options.process.cancellation, cancellation.clone());
        let result = self.execute_observation_inner(plan, cancellation);
        self.options.process.cancellation = previous;
        result
    }
    fn execute_observation_inner(
        &mut self,
        plan: &ObservationPlan,
        cancellation: inspection::Cancellation,
    ) -> Result<Observation> {
        let mut stored = self.observation(&plan.id)?;
        if stored.session != self.session
            || stored.value.phase != ObservationPhase::Proposed
            || serde_json::to_vec(&stored.value.plan)? != serde_json::to_vec(plan)?
        {
            return Err(stale());
        }
        let mut options = self.options.process.clone();
        options.cancellation = cancellation;
        options.max_output = 1024 * 1024;
        stored.value.phase = ObservationPhase::Running;
        self.persist_observation(&stored)?;
        let result = (|| {
            let _common = crate::locking::ExclusiveLock::acquire(
                File::open(plan.metadata.common_dir.to_path()?)?,
                "Shared repository is write-locked",
            )?;
            let url = self.revalidate_transport(plan, &options)?;
            let (_, home, repository) = self.initialize_transport(plan, &options)?;
            let mut probe = command(plan, &repository, &home)?;
            probe.args([
                "ls-remote",
                "--refs",
                "--exit-code",
                "--",
                &url,
                &plan.reference,
            ]);
            let output = process::transport(probe, &options, || Ok(()))?;
            if output.status.code() == Some(2) && output.stdout.is_empty() {
                return Ok(None);
            }
            if !output.status.success() {
                return Err(fail(ErrorKind::InspectionFailed,"Remote observation failed (transport, authentication, certificate, redirect or server rejection); no raw server diagnostics are retained"));
            }
            let line = inspection::output_text(output.stdout)?;
            let (oid, reference) = line
                .split_once('\t')
                .ok_or_else(|| fail(ErrorKind::InvalidData, "Invalid remote reference response"))?;
            if reference != plan.reference {
                return Err(fail(
                    ErrorKind::InvalidData,
                    "Remote returned an unexpected or ambiguous reference",
                ));
            }
            ObjectId::parse(oid, &plan.metadata.object_format).map(Some)
        })();
        match result {
            Ok(oid) => {
                stored.value.phase = ObservationPhase::Complete;
                stored.value.oid = oid;
                stored.value.observed_at_ms = Some(now_ms());
            }
            Err(error) => {
                stored.value.phase = if error.kind == ErrorKind::Cancelled {
                    ObservationPhase::Cancelled
                } else {
                    ObservationPhase::Blocked
                };
                stored.value.issue = Some(error.into());
            }
        }
        self.persist_observation(&stored)?;
        Ok(stored.value)
    }
    pub(super) fn revalidate_transport(
        &mut self,
        plan: &ObservationPlan,
        options: &ReadOptions,
    ) -> Result<String> {
        policy::preflight(&plan.metadata, &self.options)?;
        let config = policy::effective(&plan.metadata, SigningPreference::Unsigned, &self.options)?;
        self.trusted(&plan.worktree, &plan.metadata, &config)?;
        let resolved = remote::resolve_for_observation(
            &plan.metadata.root.to_path()?,
            self.options.global_config.as_deref(),
            &plan.remote,
            options,
        )?;
        if resolved.inspection.config_digest != plan.config_digest
            || resolved.inspection.generation != plan.generation
            || resolved.transport != plan.transport
        {
            return Err(stale());
        }
        if plan.transport == Transport::Ssh {
            super::ssh_transport::validate_destination(&resolved.url)?;
            let current = super::ssh_transport::bindings(self.options.remote.ssh.as_ref())?;
            let expected = plan
                .tools
                .iter()
                .filter(|b| b.role.starts_with("ssh_"))
                .cloned()
                .collect::<Vec<_>>();
            if current != expected {
                return Err(stale());
            }
        }
        if plan.transport == Transport::Https {
            let actual = self
                .options
                .remote
                .https_credentials
                .as_ref()
                .map(|p| crate::credentials::scope(&resolved.url, &p.account))
                .transpose()?;
            if actual != plan.authentication {
                return Err(stale());
            }
            if let Some(policy) = self.options.remote.https_credentials.as_ref() {
                let expected = plan
                    .tools
                    .iter()
                    .filter(|t| t.role.starts_with("credential_"))
                    .cloned()
                    .collect::<Vec<_>>();
                if crate::credentials::bindings(policy)? != expected {
                    return Err(stale());
                }
            }
        }
        validate_tools(plan)?;
        let url = if let Some(expected) = &plan.local_target {
            let target = local_target(
                &plan.metadata.root.to_path()?,
                &resolved.url,
                &self.options.remote,
            )?;
            if NativePath::from_path(&target) != *expected
                || Some(inspection::fs_identity(&target)?) != plan.local_binding
            {
                return Err(stale());
            }
            url::Url::from_file_path(target)
                .map_err(|_| unsupported("Invalid fixture URL"))?
                .to_string()
        } else {
            resolved.url
        };
        Ok(url)
    }
    pub(super) fn initialize_transport(
        &self,
        plan: &ObservationPlan,
        options: &ReadOptions,
    ) -> Result<(PathBuf, PathBuf, PathBuf)> {
        let base = self.workspace.data_directory().join("network");
        if !base.exists() {
            store::private_dir(&base)?;
        }
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::symlink_metadata(&base)?;
        if !metadata.is_dir() || metadata.is_symlink() || metadata.permissions().mode() & 0o077 != 0
        {
            return Err(unsupported("Unsafe network artifact directory"));
        }
        let directory = base.join(&plan.id);
        store::private_dir(&directory)?;
        let home = directory.join("home");
        store::private_dir(&home)?;
        if plan.transport == Transport::Ssh {
            super::ssh_transport::snapshot(&plan.tools, &home)?;
        }
        let repository = directory.join("transport.git");
        let mut init = command(plan, &directory, &home)?;
        init.args([
            "init",
            "--bare",
            "-q",
            "--object-format",
            if plan.metadata.object_format == ObjectFormat::Sha1 {
                "sha1"
            } else {
                "sha256"
            },
        ])
        .arg(&repository);
        if !process::bounded_input(init, options, None)?
            .status
            .success()
        {
            return Err(fail(
                ErrorKind::InspectionFailed,
                "Cannot initialize isolated transport repository",
            ));
        }
        Ok((directory, home, repository))
    }
    pub(super) fn completed_observation(
        &mut self,
        id: &str,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
    ) -> Result<Observation> {
        let stored = self.observation(id)?;
        if stored.value.plan.workspace != *workspace
            || stored.value.plan.worktree != *worktree
            || stored.value.phase != ObservationPhase::Complete
            || stored.value.oid.is_none()
            || stored.value.observed_at_ms.is_none()
        {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Choose a completed observation of an existing branch in this worktree",
            ));
        }
        Ok(stored.value)
    }
    pub(crate) fn observation_audit_detail(&self, payload: &str) -> Result<Observation> {
        let stored: StoredObservation = serde_json::from_str(payload)?;
        if stored.version != 1 {
            return Err(fail(
                ErrorKind::NewerSchema,
                "Unsupported observation record",
            ));
        }
        Ok(stored.value)
    }
    pub(crate) fn observations(
        &mut self,
        workspace: &WorkspaceId,
        worktree: Option<&WorktreeId>,
        offset: usize,
    ) -> Result<ObservationPage> {
        if offset > 1_000_000 {
            return Err(fail(
                ErrorKind::InvalidRequest,
                "Observation offset limit exceeded",
            ));
        }
        let (payloads, more) = {
            let mut statement = self.workspace.connection().prepare("SELECT value FROM state WHERE key LIKE 'remote-observation:%' AND json_extract(value,'$.value.plan.workspace')=?1 AND (?2 IS NULL OR json_extract(value,'$.value.plan.worktree')=?2) ORDER BY json_extract(value,'$.value.plan.created_at_ms') DESC,key LIMIT 51 OFFSET ?3")?;
            let rows = statement.query_map(
                params![workspace.0, worktree.map(|id| &id.0), offset as i64],
                |r| r.get::<_, String>(0),
            )?;
            let mut payloads = Vec::new();
            let mut size = 0;
            let mut more = false;
            for payload in rows {
                let payload = payload?;
                if payload.len() > 1024 * 1024 {
                    return Err(fail(
                        ErrorKind::OutputLimit,
                        "Stored observation exceeds 1 MiB",
                    ));
                }
                if payloads.len() == 50 || size + payload.len() > 4 * 1024 * 1024 {
                    more = true;
                    break;
                }
                size += payload.len();
                payloads.push(payload);
            }
            (payloads, more)
        };
        let mut items = Vec::new();
        for payload in payloads {
            let mut stored: StoredObservation = serde_json::from_str(&payload)?;
            if stored.version != 1 || stored.value.plan.workspace != *workspace {
                return Err(fail(
                    ErrorKind::NewerSchema,
                    "Invalid remote observation record",
                ));
            }
            if stored.value.phase == ObservationPhase::Running
                || (stored.value.phase == ObservationPhase::Proposed
                    && stored.session != self.session)
            {
                stored.value.phase = ObservationPhase::Interrupted;
            }
            items.push(stored.value);
        }
        let next = more.then_some(offset + items.len());
        Ok(ObservationPage { items, next })
    }
}
